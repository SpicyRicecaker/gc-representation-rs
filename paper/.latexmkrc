@default_files = ('index');
$common_opts = '-shell-escape -interaction=nonstopmode -file-line-error';

$pdf_mode = 4;
$lualatex = "lualatex $common_opts %O %P";
$postscript_mode = $dvi_mode = 0;

$out_dir = 'out';