#!/usr/bin/gnuplot -c

# expects data in the format produced by ./cri_to_gp_data.py

if(ARGC!=2){
  exit error "Received ".ARGC." args, expected 2 args: {data file} {1-based column to normalize by)}"
}

filename=ARG1
basis_column=ARG2+0

set datafile separator tab
set style data histogram
set style histogram cluster gap 2
set terminal png noenhanced size 2048,1536

set xlabel "Inputs"
set ylabel "Normalized Throughput"

set grid ytics

set style fill solid border rgb "black"
set auto x
set xtics rotate by -45
set yrange [0:*]

stats filename u 0 nooutput
max_col=STATS_columns
max_basis_column=(max_col-2)/3+1

# if there's at least one impl, 
if (max_col < 4 | (max_col-1) % 3 != 0) {
  exit error sprintf("got %d columns, malformed data file?\nExpected 1 + 3x columns, where x is the # of implementations and x >= 1", max_col)
}

if(basis_column < 1 | max_col < basis_column) {
  exit error sprintf("the given basis column (%d) is out of range (1 - %d)!\nValue should be index of the implementation, ignoring the low/high columns", basis_column, max_basis_column)
}
# if user passed in 1 for first impl, corresponds to column 3
basis_column=(basis_column-1) * 3 + 2 + 1


# column(i)   : low
# ''    (i+1) : mid
# ''    (i+2) : high
# syntax:
# using {y}:{xtic(column to take title for cluster from)} title col(i+1) # this is the title for the column,taken from the header
plot for [i=2:max_col:3] filename using (column(i+1)/column(basis_column)):xtic(1) title col(i+1)

