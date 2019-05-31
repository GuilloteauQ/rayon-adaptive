
set terminal png size 800,600 enhanced

do for [desc in "random sorted reversed"] {

    filename='speedup_block_size_'.desc.'.dat'
    set output 'speedup_'.desc.'.png'
    set title "Generator: ".desc
    set xlabel "Block size"
    set ylabel "Sorting time"
    set grid
    set key left top

    thread=1
    do for [thread=1:32] {
    plot filename using (($1==thread)?$2:1/0):($3) with linespoints title "Thread pool size: ".thread linecolor thread, \
    }
    # filename using (($1==2)?$2:1/0):($3) with linespoints title "Thread pool size: 2" linecolor 2, \
    # filename using (($1==3)?$2:1/0):($3) with linespoints title "Thread pool size: 3" linecolor 3, \
    # filename using (($1==4)?$2:1/0):($3) with linespoints title "Thread pool size: 4" linecolor 4, \
    # filename using (($1==5)?$2:1/0):($3) with linespoints title "Thread pool size: 5" linecolor 5, \
    # filename using (($1==6)?$2:1/0):($3) with linespoints title "Thread pool size: 6" linecolor 6, \
    # filename using (($1==7)?$2:1/0):($3) with linespoints title "Thread pool size: 7" linecolor 7, \
    # filename using (($1==8)?$2:1/0):($3) with linespoints title "Thread pool size: 8" linecolor 8, \
    # filename using (($1==9)?$2:1/0):($3) with linespoints title "Thread pool size: 9" linecolor 9, \
    # filename using (($1==10)?$2:1/0):($3) with linespoints title "Thread pool size: 10"  linecolor 10, \
    # filename using (($1==11)?$2:1/0):($3) with linespoints title "Thread pool size: 11"  linecolor 11, \
    # filename using (($1==12)?$2:1/0):($3) with linespoints title "Thread pool size: 12"  linecolor 12, \
    # filename using (($1==13)?$2:1/0):($3) with linespoints title "Thread pool size: 13"  linecolor 13, \
    # filename using (($1==14)?$2:1/0):($3) with linespoints title "Thread pool size: 14"  linecolor 14, \
    # filename using (($1==15)?$2:1/0):($3) with linespoints title "Thread pool size: 15"  linecolor 15, \
    # filename using (($1==16)?$2:1/0):($3) with linespoints title "Thread pool size: 16"  linecolor 16, \
    # filename using (($1==17)?$2:1/0):($3) with linespoints title "Thread pool size: 17"  linecolor 17, \
    # filename using (($1==18)?$2:1/0):($3) with linespoints title "Thread pool size: 18"  linecolor 18, \
    # filename using (($1==19)?$2:1/0):($3) with linespoints title "Thread pool size: 19"  linecolor 19, \
    # filename using (($1==20)?$2:1/0):($3) with linespoints title "Thread pool size: 20"  linecolor 20, \
    # filename using (($1==21)?$2:1/0):($3) with linespoints title "Thread pool size: 21"  linecolor 21, \
    # filename using (($1==22)?$2:1/0):($3) with linespoints title "Thread pool size: 22"  linecolor 22, \
    # filename using (($1==23)?$2:1/0):($3) with linespoints title "Thread pool size: 23"  linecolor 23, \
    # filename using (($1==24)?$2:1/0):($3) with linespoints title "Thread pool size: 24"  linecolor 24, \
    # filename using (($1==25)?$2:1/0):($3) with linespoints title "Thread pool size: 25"  linecolor 25, \
    # filename using (($1==26)?$2:1/0):($3) with linespoints title "Thread pool size: 26"  linecolor 26, \
    # filename using (($1==27)?$2:1/0):($3) with linespoints title "Thread pool size: 27"  linecolor 27, \
    # filename using (($1==28)?$2:1/0):($3) with linespoints title "Thread pool size: 28"  linecolor 28, \
    # filename using (($1==29)?$2:1/0):($3) with linespoints title "Thread pool size: 29"  linecolor 29, \
    # filename using (($1==30)?$2:1/0):($3) with linespoints title "Thread pool size: 30"  linecolor 30, \
    # filename using (($1==32)?$2:1/0):($3) with linespoints title "Thread pool size: 32"  linecolor 32, \

}
