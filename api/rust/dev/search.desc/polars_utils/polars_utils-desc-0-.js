searchState.loadedDescShard("polars_utils", 0, "Copy pasted from std::cell::SyncUnsafeCell can be removed …\nCentralized Polars serialization entry.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGet mutable references to several items of the Arena\nSafety\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nFind the index of the first element of <code>arr</code> that is greater …\nFind the index of the first element of <code>arr</code> that is greater …\nA cached function that use <code>LruCache</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the old value, if any.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nHyperLogLog in Practice: Algorithmic Engineering of a …\nReturns the argument unchanged.\nAdd a new hash to the sketch.\nCalls <code>U::from(self)</code>.\n<code>UnsafeCell</code>, but <code>Sync</code>.\nCreates an <code>SyncUnsafeCell</code>, with the <code>Default</code> value for T.\nReturns the argument unchanged.\nCreates a new <code>SyncUnsafeCell&lt;T&gt;</code> containing the given value.\nGets a mutable pointer to the wrapped value.\nReturns a mutable reference to the underlying data.\nCalls <code>U::from(self)</code>.\nUnwraps the value.\nConstructs a new instance of <code>SyncUnsafeCell</code> which will …\nGets a mutable pointer to the wrapped value.\nA copy of the <code>std::slice::Chunks</code> that exposes the inner …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nContains the error value\nContains the success value\nUtility whose Display impl truncates the string unless …\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nA ring-buffer with a size determined at creation-time\nGet a reference to all elements in the form of two slices.\nAdd at most <code>num</code> items of <code>value</code> into the <code>FixedRingBuffer</code>\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nPop an item at the front of the <code>FixedRingBuffer</code>\nPush an item into the <code>FixedRingBuffer</code>\nSafety\nContains a byte slice and a precomputed hash for that …\nReturns the argument unchanged.\nReturns the argument unchanged.\nConverts a hash to a partition. It is guaranteed that the …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCreates a new hash partitioner with the given number of …\nAn IndexMap where the keys are always u8 slices which are …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGets the hash, key and value at the given index by …\nGets the hash, key and value at the given index by …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nIterates over the (hash, key) pairs in insertion order.\nIterates over the values in insertion order.\nAn IndexMap where the keys are hashed and compared with …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGets the key and value at the given index by insertion …\nGets the key and value at the given index by insertion …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nIterates over the keys in insertion order.\nIterates over the values in insertion order.\nReverses indexing direction\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nSafety\nPanics\nA type logically equivalent to <code>Vec&lt;T&gt;</code>, but which does not …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nSafety\nReturns the argument unchanged.\nReturns the argument unchanged.\nSafety\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nUtility extension trait of iterator methods.\nEquivalent to <code>.collect::&lt;Vec&lt;_&gt;&gt;()</code>.\nEquivalent to <code>.collect::&lt;Result&lt;_, _&gt;&gt;()</code>.\nEquivalent to <code>.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()</code>.\nAn iterator that yields the current count and the element …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nOverflow Behavior\nSafety\nForcibly reads at least one byte each page.\nGet the configured memory prefetch function.\n<code>madvise()</code> with <code>MADV_POPULATE_READ</code> on linux systems. This a …\n<code>madvise()</code> with <code>MADV_SEQUENTIAL</code> on unix systems. This is a …\n<code>madvise()</code> with <code>MADV_WILLNEED</code> on unix systems. This is a …\nAttempt to prefetch the memory in the slice to the L2 …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nA cursor over a <code>MemSlice</code>.\nA read-only reference to a slice of memory that can …\nA handle to an immutable memory mapped buffer.\nAdvise OS how this memory map will be accessed.\nAdvise OS how this range of memory map will be accessed.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nConstruct a <code>MemSlice</code> from <code>bytes::Bytes</code>. This is zero-copy.\nConstruct a <code>MemSlice</code> from <code>bytes::Bytes</code>. This is zero-copy.\nslice outlives the returned <code>MemSlice</code>.\nConstruct a <code>MemSlice</code> that simply wraps around a <code>&amp;[u8]</code>.\nConstruct a <code>MemSlice</code> from an existing <code>Vec&lt;u8&gt;</code>. This is …\nConstruct a <code>MemSlice</code> from an existing <code>Vec&lt;u8&gt;</code>. This is …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nLock the whole memory map into RAM. Only supported on Unix.\nTransition the memory map to be writable.\nCreates a read-only memory map backed by a file.\nAttempt to prefetch the memory belonging to to this …\nAdjust the size of the memory mapping.\nPanics\nCopy the contents into a new owned <code>Vec</code>\nAdvise OS how this memory map will be accessed.\nAdvise OS how this range of memory map will be accessed.\nUnlock the whole memory map. Only supported on Unix.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nMainly used to enable compression when serializing the …\nPotentially avoids copying memory compared to a naive …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nString type that inlines small strings.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nA pair which is ordered exclusively by the first element.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\n[minor, micro]\nSerialization wrapper for T: TrySerializeToBytes that …\nWrapper around PyObject from pyo3 with additional trait …\nSerializes a Python object without additional system …\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nA cache for compiled regular expressions.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nSafety\nOr zero\nReturns the end position of the slice (offset + len).\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns the equivalent slice to apply from an offsetted …\nRestricts the bounds of the slice to within a number of …\nThis is a perfect sort particularly useful for an arg_sort …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nUtility that allows use to send pointers to another thread.\nSafety\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nSafety\nStartup system is expensive, so we do it once\nThis call is quite expensive, cache the results.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nThis elides creating a <code>TotalOrdWrap</code> for types that don’t …\nAlternative trait for Eq. By consistently using this we …\nAlternative trait for Hash. By consistently using this we …\nAlternative trait for Ord. By consistently using this we …\nConverts an f32 into a canonical form, where -0 == 0 and …\nConverts an f64 into a canonical form, where -0 == 0 and …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nFill current allocation if &gt; 0 otherwise realloc\nPerform an in-place <code>Iterator::filter_map</code> over two vectors …\nWill push an item and not check if there is enough capacity")