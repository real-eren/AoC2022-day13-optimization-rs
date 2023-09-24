#!/usr/bin/env python3

"""
Parses the output of a Rust `criterion` benchmark run and outputs a TSV with the throughput* values.
Assumes input of the form:
```
Day13_A/input_handling_baseline/right_longer
                        time:   [154.47 ns 154.77 ns 155.09 ns]
                        thrpt:  [2.9544 GiB/s 2.9606 GiB/s 2.9664 GiB/s]
```
which can be achieved by invoking the criterion bench as below:
```
cargo bench --bench day13_impls -- --format terse --quiet > data.txt
```

* The time values can also be output by slightly modifying the script.
"""

from typing import TypeAlias

UnitIndex: TypeAlias = int
Value: TypeAlias = tuple[float, UnitIndex]
Field: TypeAlias = tuple[Value, Value, Value]
TimeField: TypeAlias = Field
ThrptField: TypeAlias = Field

ImplName: TypeAlias = str
InputName: TypeAlias = str


def body(f):
    # an absurdly slow algorithm. But the input is fairly small
    """"""

    time_units = ["ps", "ns", "µs", "ms", "s"]
    thrpt_units = ["B/s", "KiB/s", "MiB/s", "GiB/s", "TiB/s"]

    def do_field(name, input, expected_prefix, unit_list) -> Field:
        prefix, _, val_list = input.partition("[")
        if prefix.strip() != expected_prefix:
            raise ValueError(f"expected prefix '{expected_prefix}', got '{prefix}'")

        if val_list[-1] != "]":
            raise ValueError(f"expected closing bracket at end of {input} for {name}")

        vals = val_list[0:-1].split(" ", maxsplit=6 + 1)
        low = (float(vals[0]), unit_list.index(vals[1]))
        mid = (float(vals[2]), unit_list.index(vals[3]))
        upp = (float(vals[4]), unit_list.index(vals[5]))

        return (low, mid, upp)

    benches: list[tuple[ImplName, InputName, TimeField, ThrptField]] = []

    input_encounter_order: list[InputName] = []
    impl_encounter_order: list[ImplName] = []

    # read all the lines in the input, parse into tuples
    # along the way, record the encounter order
    while True:
        header, time, throughput = (
            f.readline().strip(),
            f.readline().strip(),
            f.readline().strip(),
        )
        if (not header) or (not time) or (not throughput):
            break

        header_chunks: list = header.split("/", maxsplit=3 + 1)
        if len(header_chunks) != 3:
            raise ValueError(
                f"got '{header_chunks}', expected 3 fields separated by '/'"
            )

        benchmark_name, impl_name, input_name = header_chunks
        if impl_name not in impl_encounter_order:
            impl_encounter_order.append(impl_name)
        if input_name not in input_encounter_order:
            input_encounter_order.append(input_name)

        fields = (
            ("time", time, "time:", time_units),
            ("throughput", throughput, "thrpt:", thrpt_units),
        )
        time, throughput = (do_field(*field) for field in fields)
        benches.append((impl_name, input_name, time, throughput))

    scaling_list = [1024**i for i in range(len(time_units))]
    trials: dict[InputName, dict[ImplName, tuple[TimeField, ThrptField]]] = {
        name: {} for name in input_encounter_order
    }
    for impl_name, input_name, time_field, thrpt_field in benches:
        trials[input_name][impl_name] = (time_field, thrpt_field)

    min_time_unit_for_input: dict[InputName, UnitIndex] = {
        input_name: min(p[0][0][1] for p in trials[input_name].values())
        for input_name in trials
    }
    min_thrpt_unit_for_input: dict[InputName, UnitIndex] = {
        input_name: min(p[1][0][1] for p in trials[input_name].values())
        for input_name in trials
    }

    # output data file
    # https://stackoverflow.com/questions/41942693/clustered-bar-plot-in-gnuplot-with-errorbars?noredirect=1&lq=1
    def stringify(field: Field, lowest_unit: UnitIndex) -> str:
        (low, med, hi) = field
        return "\t".join(
            str(f * scaling_list[unit_index - lowest_unit])
            for (f, unit_index) in (low, med, hi)
        )

    print("Title", *(f"low\t{impl}\thigh" for impl in impl_encounter_order), sep="\t")
    for input_name in input_encounter_order:
        a: dict[ImplName, tuple[TimeField, ThrptField]] = trials[input_name]
        unit_for_input = thrpt_units[min_thrpt_unit_for_input[input_name]]
        print(
            f"{input_name} ({unit_for_input})",
            *[
                stringify(a[impl][1], min_thrpt_unit_for_input[input_name])
                for impl in impl_encounter_order
            ],
            sep="\t",
            flush=False,
        )


if __name__ == "__main__":
    import sys

    body(sys.stdin)
