use std::any::Any;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use polars_core::config::verbose;
use polars_core::error::PolarsResult;
use polars_core::frame::DataFrame;
use polars_core::prelude::{AnyValue, SchemaRef, Series, SortOptions};
use polars_core::utils::accumulate_dataframes_vertical_unchecked;
use polars_plan::prelude::SortArguments;

use crate::executors::sinks::io::{block_thread_until_io_thread_done, IOThread};
use crate::executors::sinks::memory::MemTracker;
use crate::executors::sinks::sort::ooc::sort_ooc;
use crate::operators::{DataChunk, FinalizedSink, PExecutionContext, Sink, SinkResult};
use crate::pipeline::{morsels_per_sink, FORCE_OOC_SORT};

pub struct SortSink {
    schema: SchemaRef,
    chunks: VecDeque<DataFrame>,
    // Stores available memory in the system at the start of this sink.
    // and stores the memory used by this this sink.
    mem_track: MemTracker,
    // sort in-memory or out-of-core
    ooc: bool,
    // when ooc, we write to disk using an IO thread
    // RwLock as we want to have multiple readers at once.
    io_thread: Arc<RwLock<Option<IOThread>>>,
    // location in the dataframe of the columns to sort by
    sort_idx: usize,
    sort_args: SortArguments,
    // sampled values so we can find the distribution.
    dist_sample: Vec<AnyValue<'static>>,
}

impl SortSink {
    pub(crate) fn new(sort_idx: usize, sort_args: SortArguments, schema: SchemaRef) -> Self {
        // for testing purposes
        let ooc = std::env::var(FORCE_OOC_SORT).is_ok();
        let n_morsels_per_sink = morsels_per_sink();

        let mut out = Self {
            schema,
            chunks: Default::default(),
            mem_track: MemTracker::new(n_morsels_per_sink),
            ooc,
            io_thread: Default::default(),
            sort_idx,
            sort_args,
            dist_sample: vec![],
        };
        if ooc {
            eprintln!("OOC sort forced");
            out.init_ooc().unwrap();
        }
        out
    }

    fn init_ooc(&mut self) -> PolarsResult<()> {
        if verbose() {
            eprintln!("OOC sort started");
        }
        self.ooc = true;

        // start IO thread
        let mut iot = self.io_thread.write().unwrap();
        if iot.is_none() {
            *iot = Some(IOThread::try_new(self.schema.clone(), "sort")?)
        }
        Ok(())
    }

    fn store_chunk(&mut self, chunk: DataChunk) -> PolarsResult<()> {
        if !self.ooc {
            let chunk_bytes = chunk.data.estimated_size();
            let used = self.mem_track.fetch_add(chunk_bytes);
            let free = self.mem_track.get_available();

            // we need some free memory to be able to sort
            // so we keep 3x the sort data size before we go out of core
            if used * 3 > free {
                self.init_ooc()?;
            }
        }
        self.chunks.push_back(chunk.data);
        Ok(())
    }

    fn dump(&mut self) -> PolarsResult<()> {
        // take from the front so that sorted data remains sorted in writing order
        while let Some(df) = self.chunks.pop_front() {
            if df.height() > 0 {
                // safety: we just asserted height > 0
                let sample = unsafe {
                    let s = &df.get_columns()[self.sort_idx];
                    s.to_physical_repr().get_unchecked(0).into_static().unwrap()
                };
                self.dist_sample.push(sample);

                let iot = self.io_thread.read().unwrap();
                let iot = iot.as_ref().unwrap();
                iot.dump_chunk(df)
            }
        }
        Ok(())
    }
}

impl Sink for SortSink {
    fn sink(&mut self, _context: &PExecutionContext, chunk: DataChunk) -> PolarsResult<SinkResult> {
        self.store_chunk(chunk)?;

        if self.ooc {
            self.dump()?;
        }
        Ok(SinkResult::CanHaveMoreInput)
    }

    fn combine(&mut self, other: &mut dyn Sink) {
        let other = other.as_any().downcast_mut::<Self>().unwrap();
        self.chunks.extend(std::mem::take(&mut other.chunks));
        self.ooc |= other.ooc;
        self.dist_sample
            .extend(std::mem::take(&mut other.dist_sample));

        if self.ooc {
            self.dump().unwrap()
        }
    }

    fn split(&self, _thread_no: usize) -> Box<dyn Sink> {
        Box::new(Self {
            schema: self.schema.clone(),
            chunks: Default::default(),
            mem_track: self.mem_track.clone(),
            ooc: self.ooc,
            io_thread: self.io_thread.clone(),
            sort_idx: self.sort_idx,
            sort_args: self.sort_args.clone(),
            dist_sample: vec![],
        })
    }

    fn finalize(&mut self, _context: &PExecutionContext) -> PolarsResult<FinalizedSink> {
        if self.ooc {
            let lock = self.io_thread.read().unwrap();
            let io_thread = lock.as_ref().unwrap();

            let dist = Series::from_any_values("", &self.dist_sample, false).unwrap();
            let dist = dist.sort_with(SortOptions {
                descending: self.sort_args.descending[0],
                nulls_last: self.sort_args.nulls_last,
                multithreaded: true,
            });

            block_thread_until_io_thread_done(io_thread);

            sort_ooc(
                io_thread,
                dist,
                self.sort_idx,
                self.sort_args.descending[0],
                self.sort_args.slice,
            )
        } else {
            let chunks = std::mem::take(&mut self.chunks);
            let df = accumulate_dataframes_vertical_unchecked(chunks);
            let df = sort_accumulated(
                df,
                self.sort_idx,
                self.sort_args.descending[0],
                self.sort_args.slice,
            )?;
            Ok(FinalizedSink::Finished(df))
        }
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn fmt(&self) -> &str {
        "sort"
    }
}

pub(super) fn sort_accumulated(
    df: DataFrame,
    sort_idx: usize,
    descending: bool,
    slice: Option<(i64, usize)>,
) -> PolarsResult<DataFrame> {
    let sort_column = df.get_columns()[sort_idx].clone();
    df.sort_impl(vec![sort_column], vec![descending], false, slice, true)
}
