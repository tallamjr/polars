from __future__ import annotations

import re
from itertools import zip_longest
from typing import TYPE_CHECKING, Any, Mapping, Sequence, overload

from polars import internals as pli
from polars.datatypes import (
    N_INFER_DEFAULT,
    List,
    Object,
    Struct,
)
from polars.dependencies import _PYARROW_AVAILABLE
from polars.dependencies import pandas as pd
from polars.dependencies import pyarrow as pa
from polars.exceptions import NoDataError
from polars.utils.decorators import deprecate_nonkeyword_arguments, deprecated_alias
from polars.utils.various import _cast_repr_strings_with_schema

if TYPE_CHECKING:
    from polars.dataframe import DataFrame
    from polars.dependencies import numpy as np
    from polars.series import Series
    from polars.type_aliases import Orientation, SchemaDefinition, SchemaDict


@deprecated_alias(columns="schema")
def from_dict(
    data: Mapping[str, Sequence[object] | Mapping[str, Sequence[object]] | Series],
    schema: SchemaDefinition | None = None,
    *,
    schema_overrides: SchemaDict | None = None,
) -> DataFrame:
    """
    Construct a DataFrame from a dictionary of sequences.

    This operation clones data, unless you pass a ``{str: pl.Series,}`` dict.

    Parameters
    ----------
    data : dict of sequences
        Two-dimensional data represented as a dictionary. dict must contain
        Sequences.
    schema : Sequence of str, (str,DataType) pairs, or a {str:DataType,} dict
        The DataFrame schema may be declared in several ways:

        * As a dict of {name:type} pairs; if type is None, it will be auto-inferred.
        * As a list of column names; in this case types are automatically inferred.
        * As a list of (name,type) pairs; this is equivalent to the dictionary form.

        If you supply a list of column names that does not match the names in the
        underlying data, the names given here will overwrite them. The number
        of names given in the schema should match the underlying data dimensions.
    schema_overrides : dict, default None
        Support type specification or override of one or more columns; note that
        any dtypes inferred from the columns param will be overridden.

    Returns
    -------
    :class:`DataFrame`

    Examples
    --------
    >>> df = pl.from_dict({"a": [1, 2], "b": [3, 4]})
    >>> df
    shape: (2, 2)
    ┌─────┬─────┐
    │ a   ┆ b   │
    │ --- ┆ --- │
    │ i64 ┆ i64 │
    ╞═════╪═════╡
    │ 1   ┆ 3   │
    │ 2   ┆ 4   │
    └─────┴─────┘

    """
    return pli.DataFrame._from_dict(
        data, schema=schema, schema_overrides=schema_overrides
    )


@deprecate_nonkeyword_arguments(allowed_args=["data", "schema"])
@deprecated_alias(dicts="data", stacklevel=4)
def from_dicts(
    data: Sequence[dict[str, Any]],
    infer_schema_length: int | None = N_INFER_DEFAULT,
    *,
    schema: SchemaDefinition | None = None,
    schema_overrides: SchemaDict | None = None,
) -> DataFrame:
    """
    Construct a DataFrame from a sequence of dictionaries. This operation clones data.

    Parameters
    ----------
    data
        Sequence with dictionaries mapping column name to value
    infer_schema_length
        How many dictionaries/rows to scan to determine the data types
        if set to `None` then ALL dicts are scanned; this will be slow.
    schema : Sequence of str, (str,DataType) pairs, or a {str:DataType,} dict
        The DataFrame schema may be declared in several ways:

        * As a dict of {name:type} pairs; if type is None, it will be auto-inferred.
        * As a list of column names; in this case types are automatically inferred.
        * As a list of (name,type) pairs; this is equivalent to the dictionary form.

        If a list of column names is supplied that does NOT match the names in the
        underlying data, the names given here will overwrite the actual fields in
        the order that they appear - however, in this case it is typically clearer
        to rename after loading the frame.

        If you want to drop some of the fields found in the input dictionaries, a
        _partial_ schema can be declared, in which case omitted fields will not be
        loaded. Similarly you can extend the loaded frame with empty columns by adding
        them to the schema.
    schema_overrides : dict, default None
        Support override of inferred types for one or more columns.

    Returns
    -------
    :class:`DataFrame`

    Examples
    --------
    >>> data = [{"a": 1, "b": 4}, {"a": 2, "b": 5}, {"a": 3, "b": 6}]
    >>> df = pl.from_dicts(data)
    >>> df
    shape: (3, 2)
    ┌─────┬─────┐
    │ a   ┆ b   │
    │ --- ┆ --- │
    │ i64 ┆ i64 │
    ╞═════╪═════╡
    │ 1   ┆ 4   │
    │ 2   ┆ 5   │
    │ 3   ┆ 6   │
    └─────┴─────┘

    Declaring a partial ``schema`` will drop the omitted columns.

    >>> df = pl.from_dicts(data, schema={"a": pl.Int32})
    >>> df
    shape: (3, 1)
    ┌─────┐
    │ a   │
    │ --- │
    │ i32 │
    ╞═════╡
    │ 1   │
    │ 2   │
    │ 3   │
    └─────┘

    Can also use the ``schema`` param to extend the loaded columns with one
    or more additional (empty) columns that are not present in the input dicts:

    >>> pl.from_dicts(
    ...     data,
    ...     schema=["a", "b", "c", "d"],
    ...     schema_overrides={"c": pl.Float64, "d": pl.Utf8},
    ... )
    shape: (3, 4)
    ┌─────┬─────┬──────┬──────┐
    │ a   ┆ b   ┆ c    ┆ d    │
    │ --- ┆ --- ┆ ---  ┆ ---  │
    │ i64 ┆ i64 ┆ f64  ┆ str  │
    ╞═════╪═════╪══════╪══════╡
    │ 1   ┆ 4   ┆ null ┆ null │
    │ 2   ┆ 5   ┆ null ┆ null │
    │ 3   ┆ 6   ┆ null ┆ null │
    └─────┴─────┴──────┴──────┘

    """
    if not data and not (schema or schema_overrides):
        raise NoDataError("No rows. Cannot infer schema.")

    return pli.DataFrame(
        data,
        schema=schema,
        schema_overrides=schema_overrides,
        infer_schema_length=infer_schema_length,
    )


@deprecated_alias(columns="schema")
def from_records(
    data: Sequence[Sequence[Any]],
    schema: SchemaDefinition | None = None,
    *,
    schema_overrides: SchemaDict | None = None,
    orient: Orientation | None = None,
    infer_schema_length: int | None = N_INFER_DEFAULT,
) -> DataFrame:
    """
    Construct a DataFrame from a sequence of sequences. This operation clones data.

    Note that this is slower than creating from columnar memory.

    Parameters
    ----------
    data : Sequence of sequences
        Two-dimensional data represented as a sequence of sequences.
    schema : Sequence of str, (str,DataType) pairs, or a {str:DataType,} dict
        The DataFrame schema may be declared in several ways:

        * As a dict of {name:type} pairs; if type is None, it will be auto-inferred.
        * As a list of column names; in this case types are automatically inferred.
        * As a list of (name,type) pairs; this is equivalent to the dictionary form.

        If you supply a list of column names that does not match the names in the
        underlying data, the names given here will overwrite them. The number
        of names given in the schema should match the underlying data dimensions.
    schema_overrides : dict, default None
        Support type specification or override of one or more columns; note that
        any dtypes inferred from the columns param will be overridden.
    orient : {None, 'col', 'row'}
        Whether to interpret two-dimensional data as columns or as rows. If None,
        the orientation is inferred by matching the columns and data dimensions. If
        this does not yield conclusive results, column orientation is used.
    infer_schema_length
        How many dictionaries/rows to scan to determine the data types
        if set to `None` all rows are scanned. This will be slow.

    Returns
    -------
    :class:`DataFrame`

    Examples
    --------
    >>> data = [[1, 2, 3], [4, 5, 6]]
    >>> df = pl.from_records(data, schema=["a", "b"])
    >>> df
    shape: (3, 2)
    ┌─────┬─────┐
    │ a   ┆ b   │
    │ --- ┆ --- │
    │ i64 ┆ i64 │
    ╞═════╪═════╡
    │ 1   ┆ 4   │
    │ 2   ┆ 5   │
    │ 3   ┆ 6   │
    └─────┴─────┘

    """
    return pli.DataFrame._from_records(
        data,
        schema=schema,
        schema_overrides=schema_overrides,
        orient=orient,
        infer_schema_length=infer_schema_length,
    )


def from_repr(tbl: str) -> DataFrame:
    """
    Debug function that reconstructs a DataFrame from a table repr.

    Parameters
    ----------
    tbl
        A string containing a polars table repr; does not need to be trimmed
        of whitespace (or leading prompts) as the table will be found/extracted
        automatically.

    Notes
    -----
    This function handles the default UTF8_FULL and UTF8_FULL_CONDENSED format
    tables (with or without rounded corners). Truncated columns/rows are omitted,
    wrapped headers are accounted for, and dtypes identified.

    Currently compound types such as List and Struct are not supported,
    (and neither is Time) though support is planned.

    Examples
    --------
    >>> df = pl.from_repr(
    ...     '''
    ...     Out[3]:
    ...     shape: (1, 5)
    ...     ┌───────────┬────────────┬───┬───────┬────────────────────────────────┐
    ...     │ source_ac ┆ source_cha ┆ … ┆ ident ┆ timestamp                      │
    ...     │ tor_id    ┆ nnel_id    ┆   ┆ ---   ┆ ---                            │
    ...     │ ---       ┆ ---        ┆   ┆ str   ┆ datetime[μs, Asia/Tokyo]       │
    ...     │ i32       ┆ i64        ┆   ┆       ┆                                │
    ...     ╞═══════════╪════════════╪═══╪═══════╪════════════════════════════════╡
    ...     │ 123456780 ┆ 9876543210 ┆ … ┆ a:b:c ┆ 2023-03-25 10:56:59.663053 JST │
    ...     │ …         ┆ …          ┆ … ┆ …     ┆ …                              │
    ...     │ 803065983 ┆ 2055938745 ┆ … ┆ x:y:z ┆ 2023-03-25 12:38:18.050545 JST │
    ...     └───────────┴────────────┴───┴───────┴────────────────────────────────┘
    ... '''
    ... )
    >>> df
    shape: (2, 4)
    ┌─────────────────┬───────────────────┬───────┬────────────────────────────────┐
    │ source_actor_id ┆ source_channel_id ┆ ident ┆ timestamp                      │
    │ ---             ┆ ---               ┆ ---   ┆ ---                            │
    │ i32             ┆ i64               ┆ str   ┆ datetime[μs, Asia/Tokyo]       │
    ╞═════════════════╪═══════════════════╪═══════╪════════════════════════════════╡
    │ 123456780       ┆ 9876543210        ┆ a:b:c ┆ 2023-03-25 10:56:59.663053 JST │
    │ 803065983       ┆ 2055938745        ┆ x:y:z ┆ 2023-03-25 12:38:18.050545 JST │
    └─────────────────┴───────────────────┴───────┴────────────────────────────────┘
    >>> df.schema
    {'source_actor_id': Int32,
     'source_channel_id': Int64,
     'ident': Utf8,
     'timestamp': Datetime(time_unit='us', time_zone='Asia/Tokyo')}

    """
    from polars.datatypes.convert import dtype_short_repr_to_dtype

    # pick dataframe table out of the given string
    m = re.search(r"([┌╭].*?[┘╯])", tbl, re.DOTALL)
    if m is None:
        raise ValueError("Table not found in the given string")

    # extract elements from table structure
    lines = m.group().split("\n")[1:-1]
    rows = [
        [re.sub(r"^[\W+]*│", "", elem).strip() for elem in row]
        for row in [re.split("[┆|]", row.rstrip("│ ")) for row in lines]
        if len(row) > 1 or not re.search("├[╌┼]+┤", row[0])
    ]

    # determine beginning/end of the header block
    table_body_start = 2
    for idx, (elem, *_) in enumerate(rows):
        if re.match(r"^\W*╞", elem):
            table_body_start = idx
            break

    # handle headers with wrapped column names and determine headers/dtypes
    header_block = ["".join(h).split("---") for h in zip(*rows[:table_body_start])]
    headers, dtypes = (list(h) for h in zip_longest(*header_block))
    body = rows[table_body_start + 1 :]

    # transpose rows into columns, detect/omit truncated columns
    coldata = list(zip(*(row for row in body if not all((e == "…") for e in row))))
    for el in ("…", "..."):
        if el in headers:
            idx = headers.index(el)
            for table_elem in (headers, dtypes):
                table_elem.pop(idx)
            if coldata:
                coldata.pop(idx)

    # init cols as utf8 Series, handle "null" -> None, create schema from repr dtype
    data = [pli.Series([(None if v == "null" else v) for v in cd]) for cd in coldata]
    schema = dict(zip(headers, (dtype_short_repr_to_dtype(d) for d in dtypes)))
    for tp in set(schema.values()):
        # TODO: handle basic compound types
        if tp in (List, Struct):
            raise NotImplementedError(
                f"'from_repr' does not (yet) support {tp} dtype columns"
            )
        elif tp == Object:
            raise ValueError("'from_repr' does not (and cannot) support Object dtype")

    # construct DataFrame from string series and cast from repr to native dtype
    df = pli.DataFrame(data=data, orient="col", schema=list(schema))
    return _cast_repr_strings_with_schema(df, schema)


@deprecated_alias(columns="schema")
def from_numpy(
    data: np.ndarray[Any, Any],
    schema: SchemaDefinition | None = None,
    *,
    schema_overrides: SchemaDict | None = None,
    orient: Orientation | None = None,
) -> DataFrame:
    """
    Construct a DataFrame from a numpy ndarray. This operation clones data.

    Note that this is slower than creating from columnar memory.

    Parameters
    ----------
    data : :class:`numpy.ndarray`
        Two-dimensional data represented as a numpy ndarray.
    schema : Sequence of str, (str,DataType) pairs, or a {str:DataType,} dict
        The DataFrame schema may be declared in several ways:

        * As a dict of {name:type} pairs; if type is None, it will be auto-inferred.
        * As a list of column names; in this case types are automatically inferred.
        * As a list of (name,type) pairs; this is equivalent to the dictionary form.

        If you supply a list of column names that does not match the names in the
        underlying data, the names given here will overwrite them. The number
        of names given in the schema should match the underlying data dimensions.
    schema_overrides : dict, default None
        Support type specification or override of one or more columns; note that
        any dtypes inferred from the columns param will be overridden.
    orient : {None, 'col', 'row'}
        Whether to interpret two-dimensional data as columns or as rows. If None,
        the orientation is inferred by matching the columns and data dimensions. If
        this does not yield conclusive results, column orientation is used.

    Returns
    -------
    :class:`DataFrame`

    Examples
    --------
    >>> import numpy as np
    >>> data = np.array([[1, 2, 3], [4, 5, 6]])
    >>> df = pl.from_numpy(data, schema=["a", "b"], orient="col")
    >>> df
    shape: (3, 2)
    ┌─────┬─────┐
    │ a   ┆ b   │
    │ --- ┆ --- │
    │ i64 ┆ i64 │
    ╞═════╪═════╡
    │ 1   ┆ 4   │
    │ 2   ┆ 5   │
    │ 3   ┆ 6   │
    └─────┴─────┘

    """
    return pli.DataFrame._from_numpy(
        data, schema=schema, orient=orient, schema_overrides=schema_overrides
    )


@deprecate_nonkeyword_arguments(allowed_args=["data", "schema"])
def from_arrow(
    data: pa.Table | pa.Array | pa.ChunkedArray,
    rechunk: bool = True,
    schema: SchemaDefinition | None = None,
    schema_overrides: SchemaDict | None = None,
) -> DataFrame | Series:
    """
    Create a DataFrame or Series from an Arrow Table or Array.

    This operation will be zero copy for the most part. Types that are not
    supported by Polars may be cast to the closest supported type.

    Parameters
    ----------
    data : :class:`pyarrow.Table` or :class:`pyarrow.Array`
        Data representing an Arrow Table or Array.
    rechunk : bool, default True
        Make sure that all data is in contiguous memory.
    schema : Sequence of str, (str,DataType) pairs, or a {str:DataType,} dict
        The DataFrame schema may be declared in several ways:

        * As a dict of {name:type} pairs; if type is None, it will be auto-inferred.
        * As a list of column names; in this case types are automatically inferred.
        * As a list of (name,type) pairs; this is equivalent to the dictionary form.

        If you supply a list of column names that does not match the names in the
        underlying data, the names given here will overwrite them. The number
        of names given in the schema should match the underlying data dimensions.
    schema_overrides : dict, default None
        Support type specification or override of one or more columns; note that
        any dtypes inferred from the schema param will be overridden.

    Returns
    -------
    :class:`DataFrame` or :class:`Series`

    Examples
    --------
    Constructing a DataFrame from an Arrow Table:

    >>> import pyarrow as pa
    >>> data = pa.table({"a": [1, 2, 3], "b": [4, 5, 6]})
    >>> df = pl.from_arrow(data)
    >>> df
    shape: (3, 2)
    ┌─────┬─────┐
    │ a   ┆ b   │
    │ --- ┆ --- │
    │ i64 ┆ i64 │
    ╞═════╪═════╡
    │ 1   ┆ 4   │
    │ 2   ┆ 5   │
    │ 3   ┆ 6   │
    └─────┴─────┘

    Constructing a Series from an Arrow Array:

    >>> import pyarrow as pa
    >>> data = pa.array([1, 2, 3])
    >>> series = pl.from_arrow(data)
    >>> series
    shape: (3,)
    Series: '' [i64]
    [
        1
        2
        3
    ]

    """
    if isinstance(data, pa.Table):
        return pli.DataFrame._from_arrow(
            data, rechunk=rechunk, schema=schema, schema_overrides=schema_overrides
        )
    elif isinstance(data, (pa.Array, pa.ChunkedArray)):
        return pli.Series._from_arrow("", data, rechunk)
    else:
        raise ValueError(f"expected Arrow Table or Array, got {type(data)}.")


@overload
def from_pandas(
    data: pd.DataFrame,
    rechunk: bool = ...,
    nan_to_null: bool = ...,
    schema_overrides: SchemaDict | None = ...,
    *,
    include_index: bool = ...,
) -> DataFrame:
    ...


@overload
def from_pandas(
    data: pd.Series | pd.DatetimeIndex,
    rechunk: bool = ...,
    nan_to_null: bool = ...,
    schema_overrides: SchemaDict | None = ...,
    *,
    include_index: bool = ...,
) -> Series:
    ...


@deprecate_nonkeyword_arguments()
@deprecated_alias(nan_to_none="nan_to_null", df="data", stacklevel=4)
def from_pandas(
    data: pd.DataFrame | pd.Series | pd.DatetimeIndex,
    rechunk: bool = True,
    nan_to_null: bool = True,
    schema_overrides: SchemaDict | None = None,
    *,
    include_index: bool = False,
) -> DataFrame | Series:
    """
    Construct a Polars DataFrame or Series from a pandas DataFrame or Series.

    This operation clones data.

    This requires that :mod:`pandas` and :mod:`pyarrow` are installed.

    Parameters
    ----------
    data: :class:`pandas.DataFrame`, :class:`pandas.Series`, :class:`pandas.DatetimeIndex`
        Data represented as a pandas DataFrame, Series, or DatetimeIndex.
    rechunk : bool, default True
        Make sure that all data is in contiguous memory.
    nan_to_null : bool, default True
        If data contains `NaN` values PyArrow will convert the ``NaN`` to ``None``
    schema_overrides : dict, default None
        Support override of inferred types for one or more columns.
    include_index : bool, default False
        Load any non-default pandas indexes as columns.

    Returns
    -------
    :class:`DataFrame`

    Examples
    --------
    Constructing a :class:`DataFrame` from a :class:`pandas.DataFrame`:

    >>> import pandas as pd
    >>> pd_df = pd.DataFrame([[1, 2, 3], [4, 5, 6]], columns=["a", "b", "c"])
    >>> df = pl.from_pandas(pd_df)
    >>> df
        shape: (2, 3)
    ┌─────┬─────┬─────┐
    │ a   ┆ b   ┆ c   │
    │ --- ┆ --- ┆ --- │
    │ i64 ┆ i64 ┆ i64 │
    ╞═════╪═════╪═════╡
    │ 1   ┆ 2   ┆ 3   │
    │ 4   ┆ 5   ┆ 6   │
    └─────┴─────┴─────┘

    Constructing a Series from a :class:`pd.Series`:

    >>> import pandas as pd
    >>> pd_series = pd.Series([1, 2, 3], name="pd")
    >>> df = pl.from_pandas(pd_series)
    >>> df
    shape: (3,)
    Series: 'pd' [i64]
    [
        1
        2
        3
    ]

    """  # noqa: W505
    if isinstance(data, (pd.Series, pd.DatetimeIndex)):
        return pli.Series._from_pandas("", data, nan_to_null=nan_to_null)
    elif isinstance(data, pd.DataFrame):
        return pli.DataFrame._from_pandas(
            data,
            rechunk=rechunk,
            nan_to_null=nan_to_null,
            schema_overrides=schema_overrides,
            include_index=include_index,
        )
    else:
        raise ValueError(f"Expected pandas DataFrame or Series, got {type(data)}.")


@deprecate_nonkeyword_arguments()
def from_dataframe(df: Any, allow_copy: bool = True) -> DataFrame:
    """
    Build a Polars DataFrame from any dataframe supporting the interchange protocol.

    Parameters
    ----------
    df
        Object supporting the dataframe interchange protocol, i.e. must have implemented
        the ``__dataframe__`` method.
    allow_copy
        Allow memory to be copied to perform the conversion. If set to False, causes
        conversions that are not zero-copy to fail.

    Notes
    -----
    Details on the dataframe interchange protocol:
    https://data-apis.org/dataframe-protocol/latest/index.html

    Zero-copy conversions currently cannot be guaranteed and will throw a
    ``RuntimeError``.

    Using a dedicated function like :func:`from_pandas` or :func:`from_arrow` is a more
    efficient method of conversion.

    """
    if isinstance(df, pli.DataFrame):
        return df
    if not hasattr(df, "__dataframe__"):
        raise TypeError(
            f"`df` of type {type(df)} does not support the dataframe interchange"
            " protocol."
        )
    if not _PYARROW_AVAILABLE or int(pa.__version__.split(".")[0]) < 11:
        raise ImportError(
            "pyarrow>=11.0.0 is required for converting a dataframe interchange object"
            " to a Polars dataframe."
        )

    import pyarrow.interchange  # noqa: F401

    pa_table = pa.interchange.from_dataframe(df, allow_copy=allow_copy)
    return from_arrow(pa_table, rechunk=allow_copy)  # type: ignore[return-value]
