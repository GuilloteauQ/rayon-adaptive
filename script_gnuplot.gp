
set title "Generator: ".desc
set xlabel "Array size"
set ylabel "Thread pool size"
set zlabel "Sort time"
set hidden3d
set dgrid3d 50,50 qnorm 2

# if (!exists("filename")) filemane='random.dat'

splot filename using 1:2:3 title "J/J" with lines linecolor 1, \
filename using 1:2:4 title "J/JC" with lines linecolor 2, \
filename using 1:2:5 title "JC/J" with lines linecolor 3, \
filename using 1:2:6 title "JC/JC" with lines linecolor 4, \
filename using 1:2:7 title "Seq" with lines linecolor 5
pause -1
