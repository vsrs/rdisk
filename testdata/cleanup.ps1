$cwd = Get-Location
$data_dir = "$cwd\testdata"

Remove-Item "$data_dir\*" -Exclude *.zip, *.ps1