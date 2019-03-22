set terminal png size 800,600 enhanced

do for [desc in "random reversed sorted"] {

    do for [policy in "J JC"] {

        filename='comp_'.desc.'_'.policy.'.dat'

        set output ''.desc.'_png/speedup_'.desc.'_'.policy.'.png'
        set title "Generator: ".desc." with policy ".policy
        set xlabel "Thread pool size"
        set ylabel "Speedup"

        plot filename using ($2):($3/$3) with linespoints title "Classic" linecolor 1, \
        filename using ($2):($4/$3) with linespoints title "Raw" linecolor 2, \
        filename using ($2):($5/$3) with linespoints title "Raw + No copy" linecolor 3

        # filename using ($2):(($4 - $3) * 100/$3) with linespoints title "Raw" linecolor 2, \
        # filename using ($2):(($5 - $3) * 100/$3) with linespoints title "Raw + No copy" linecolor 3
    }
}
