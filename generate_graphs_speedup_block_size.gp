
set terminal png size 800,600 enhanced

do for [desc in "random sorted reversed"] {

    filename='speedup_block_size_'.desc.'.dat'
    set output 'speedup_'.desc.'.png'
    set title "Generator: ".desc
    set xlabel "Block size"
    set ylabel "Sorting time"
    set grid
    # set key left top

    plot for [thread in "1 2 4 8 16 32"] filename using (($1==thread)?$2:1/0):3 with linespoints linewidth 2 pointsize 2 title "Pool size: ".thread # linecolor log(thread)/log(2) + 1

  }
