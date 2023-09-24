#!/usr/bin/gnuplot -c

# expects data in the format produced by ./cri_to_gp_data.py
filename=ARG1

set datafile separator tab
set style data histogram
set style histogram cluster errorbars gap 2 lw 1
set terminal png noenhanced size 2048,1536

set xlabel "Inputs"
set ylabel "Throughput"

set grid ytics

set style fill solid border rgb "black"
set auto x
set xtics rotate by -45
set yrange [0:*]

stats filename  u 0 nooutput
max_col = STATS_columns

# column(i)   : low
# ''    (i+1) : mid
# ''    (i+2) : high
# syntax:
# using {y}:{y.low}:{y.high}:{xtic(column to take title for cluster from)} with yerrorbars {title col(i+1) --this is the title for the column,taken from the header--}
plot for [i=2:max_col:3] filename using (column(i+1)):i:(column(i+2)):xtic(1) title col(i+1)

