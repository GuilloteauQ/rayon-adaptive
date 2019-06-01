
set terminal png size 800,600 enhanced

do for [desc in "random sorted reversed"] {

    filename='new_speedup_block_size_'.desc.'.dat'
    set output 'speedup_'.desc.'22.png'
    set title "Generator: ".desc
    set xlabel "Block size in % of the array"
    set ylabel "Speedup"
    set grid
    set logscale x
    # set key left top

    plot for [thread in "1 2 3 6 4 8 16 32"] filename using (($1==thread)?($2*100/$4):1/0):($5/$3) with linespoints pointsize 1 title "Pool size: ".thread # linecolor log(thread)/log(2) + 1

  }
