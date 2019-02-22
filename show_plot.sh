#! /bin/sh

case "$1" in
    random)
        gnuplot -e "filename='random.dat'" -e "desc='Random'" script_gnuplot.gp
        ;;
    sorted)
        gnuplot -e "filename='sorted.dat'" -e "desc='Sorted'" script_gnuplot.gp
        ;;
    rev)
        gnuplot -e "filename='reversed.dat'" -e "desc='Reversed'" script_gnuplot.gp
        ;;
    dupli)
        gnuplot -e "filename='random_with_duplicates.dat'" -e "desc='Random with duplicates'" script_gnuplot.gp
        ;;
    *)
        echo "Bad usage"
        exit 1
esac
