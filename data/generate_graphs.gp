
set terminal png size 800,600 enhanced

# do for [desc in "random sorted reversed random_with_duplicates"] {
do for [desc in "sorted"] {

    filename=''.desc.'.dat'

    do for [size in "10000 20000 50000 100000 200000 500000 1000000 5000000 10000000"] {
        set output ''.desc.'_png/'.desc.'_'.size.'.png'
        set title "Generator: ".desc." with array size ".size
        set xlabel "Thread pool size"
        set ylabel "Sort time"
        plot filename using (($1==size)?$2:1/0):3 with linespoints title "J/J" linecolor 1, \
        filename using (($1==size)?$2:1/0):4 with linespoints title "J/JC" linecolor 2, \
        filename using (($1==size)?$2:1/0):5 with linespoints title "JC/J" linecolor 3, \
        filename using (($1==size)?$2:1/0):6 with linespoints title "JC/JC" linecolor 4, \
        filename using (($1==size)?$2:1/0):8 with linespoints title "Raw J/J" linecolor 6, \
        filename using (($1==size)?$2:1/0):9 with linespoints title "Raw J/JC" linecolor 7, \
        filename using (($1==size)?$2:1/0):10 with linespoints title "Raw JC/J" linecolor 8, \
        filename using (($1==size)?$2:1/0):11 with linespoints title "Raw JC/JC" linecolor 9, \
        filename using (($1==size)?$2:1/0):12 with linespoints title "Swap J/J" linecolor 10, \
        filename using (($1==size)?$2:1/0):13 with linespoints title "Swap J/JC" linecolor 11, \
        filename using (($1==size)?$2:1/0):14 with linespoints title "Swap JC/J" linecolor 12, \
        filename using (($1==size)?$2:1/0):15 with linespoints title "Swap JC/JC" linecolor 13, \
        filename using (($1==size)?$2:1/0):7 with linespoints title "Seq" linecolor 5

        set output ''.desc.'_png/speedup_'.desc.'_'.size.'.png'
        set title "Generator: ".desc." with array size ".size
        set xlabel "Thread pool size"
        set ylabel "Speedup"
        plot filename using (($1==size)?$2:1/0):($7/$3) with linespoints title "J/J" linecolor 1, \
        filename using (($1==size)?$2:1/0):($7/$4) with linespoints title "J/JC" linecolor 2, \
        filename using (($1==size)?$2:1/0):($7/$5) with linespoints title "JC/J" linecolor 3, \
        filename using (($1==size)?$2:1/0):($7/$6) with linespoints title "JC/JC" linecolor 4, \
        filename using (($1==size)?$2:1/0):($7/$8) with linespoints title "Raw J/J" linecolor 6, \
        filename using (($1==size)?$2:1/0):($7/$9) with linespoints title "Raw J/JC" linecolor 7, \
        filename using (($1==size)?$2:1/0):($7/$10) with linespoints title "Raw JC/J" linecolor 8, \
        filename using (($1==size)?$2:1/0):($7/$11) with linespoints title "Raw JC/JC" linecolor 9, \
        filename using (($1==size)?$2:1/0):($7/$12) with linespoints title "Swap J/J" linecolor 10, \
        filename using (($1==size)?$2:1/0):($7/$13) with linespoints title "Swap J/JC" linecolor 11, \
        filename using (($1==size)?$2:1/0):($7/$14) with linespoints title "Swap JC/J" linecolor 12, \
        filename using (($1==size)?$2:1/0):($7/$15) with linespoints title "Swap JC/JC" linecolor 13, \
        filename using (($1==size)?$2:1/0):($7/$7) with linespoints title "Seq" linecolor 5

    }

}
