
set terminal png size 800,600 enhanced

do for [desc in "random sorted reversed random_with_duplicates"] {

    filename=''.desc.'.dat'

    do for [size in "10000 20000 50000 100000 200000"] {
        set output ''.desc.'_png/'.desc.'_'.size.'.png'
        set title "Generator: ".desc." with array size ".size
        set xlabel "Thread pool size"
        set ylabel "Sort time"
        plot filename using (($1==size)?$2:1/0):3 with linespoints title "J/J" linecolor 1, \
        filename using (($1==size)?$2:1/0):4 with linespoints title "J/JC" linecolor 2, \
        filename using (($1==size)?$2:1/0):5 with linespoints title "JC/J" linecolor 3, \
        filename using (($1==size)?$2:1/0):6 with linespoints title "JC/JC" linecolor 4, \
        filename using (($1==size)?$2:1/0):7 with linespoints title "Seq" linecolor 5
    }


}
