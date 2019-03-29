
set terminal png size 800,600 enhanced

do for [desc in "random sorted reversed"] {

    filename='29032019_'.desc.'.dat'

    # do for [size in "10000 20000 50000 100000 200000 500000 1000000 5000000 10000000"] {
    do for [size in "10000000"] {
        set output ''.desc.'_png/29032019_speedup_'.desc.'_'.size.'.png'
        set title "Generator: ".desc." with array size ".size
        set xlabel "Thread pool size"
        set ylabel "Speedup"

        plot filename using (($1==size)?$2:1/0):($5/$3) with linespoints title "J" linecolor 1, \
        filename using (($1==size)?$2:1/0):($5/$4) with linespoints title "JC" linecolor 2, \
        filename using (($1==size)?$2:1/0):($5/$6) with linespoints title "Raw J" linecolor 3, \
        filename using (($1==size)?$2:1/0):($5/$7) with linespoints title "Raw JC" linecolor 4, \
        filename using (($1==size)?$2:1/0):($5/$8) with linespoints title "Swap J" linecolor 6, \
        filename using (($1==size)?$2:1/0):($5/$9) with linespoints title "Swap JC" linecolor 7, \
        filename using (($1==size)?$2:1/0):($5/$10) with linespoints title "No copy J" linecolor 8, \
        filename using (($1==size)?$2:1/0):($5/$11) with linespoints title "No copy JC" linecolor 9, \
        filename using (($1==size)?$2:1/0):($5/$12) with linespoints title "Cut J" linecolor 10, \
        filename using (($1==size)?$2:1/0):($5/$13) with linespoints title "Cut JC" linecolor 11, \
        filename using (($1==size)?$2:1/0):($5/$14) with linespoints title "Rayon" linecolor 12, \
        filename using (($1==size)?$2:1/0):($5/$5) with linespoints title "Seq" linecolor 5

    }

}
