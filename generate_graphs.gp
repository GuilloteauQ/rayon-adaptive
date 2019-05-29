
set terminal png size 800,600 enhanced

do for [desc in "random sorted reversed"] {

    filename='compa_join_29052019_'.desc.'.dat'

    do for [size in "10000000"] {
        set output 'speedup_'.desc.'_'.size.'.png'
        set title "Generator: ".desc." with array size ".size
        set xlabel "Thread pool size"
        set ylabel "Speedup"
        set grid
        set key left top

        plot filename using (($1==size)?$2:1/0):($3/$4) with linespoints title "3/3 Join" linecolor 1, \
        filename using (($1==size)?$2:1/0):($3/$5) with linespoints title "2/2 Join" linecolor 2, \
        filename using (($1==size)?$2:1/0):($3/$6) with linespoints title "3/2 Join" linecolor 3, \
        filename using (($1==size)?$2:1/0):($3/$7) with linespoints title "Rayon" linecolor 4, \
        filename using (($1==size)?$2:1/0):($3/$8) with linespoints title "3/3 Join no copy" linecolor 6, \
        filename using (($1==size)?$2:1/0):($3/$3) with linespoints title "Seq" linecolor 5, \
    }

}
