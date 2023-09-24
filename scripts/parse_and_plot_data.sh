#!/bin/sh
# Commands used to generate plots
# assumes you are calling this from the root directory
# output will go to './assets/'

if [ ! -d "./assets" ]; then 
  echo >&2 "could not find './assets/' folder."
  echo >&2 "Make sure you are running this script from the root directory of this project!"
  exit 1
fi

if [ ! -f "./assets/full_raw.txt" ]; then
  echo >&2 "could not find file './assets/full_raw.txt"
  echo >&2 "Make sure you ran the './scripts/run_all_benches.sh' command prior to this script!"
  exit 1
fi

./scripts/cri_to_gp_data.py < ./assets/full_raw.txt > ./assets/full_data.data
./scripts/thrpt_bar_chart.gp ./assets/full_data.data > ./assets/full_data.png

# extract & plot data excluding the 10kb number inputs
grep -v "number" ./assets/full_data.data > ./assets/data_excluding_single_number.data
./scripts/thrpt_bar_chart.gp ./assets/data_excluding_single_number.data > ./assets/data_excluding_single_number.png

# extract data for the 10kb number inputs
head -n1 ./assets/full_data.data > ./assets/data_only_single_number.data
grep "number" ./assets/full_data.data >> ./assets/data_only_single_number.data
./scripts/thrpt_bar_chart.gp ./assets/data_only_single_number.data > ./assets/data_only_single_number.png

# extract & plot data for the 'naive' impls
grep --after-context=2 --no-group-separator "naive" ./assets/full_raw.txt | ./scripts/cri_to_gp_data.py > ./assets/data_only_naive.data
./scripts/thrpt_bar_chart.gp ./assets/data_only_naive.data > ./assets/data_only_naive.png
## plot normalized data for naive
./scripts/normalized_thrpt_bar_chart.gp ./assets/data_only_naive.data 2 > ./assets/normalized_only_naive.png
