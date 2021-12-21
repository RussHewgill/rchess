reset
set terminal png size 1000, 1000
set output 'plot2d.png'


set grid
set xtics auto
set ytics auto
# set xtics 500
# set ytics 500
set mxtics 2
set mytics 2

# set xrange [0:50]
# set yrange [0:2200]

min(x,y) = (x > y) ? y : x

# plot min((6 * x + 229) * x - 215, 2000)

plot ((21.9 + log(1)) / 2) * log(x)

