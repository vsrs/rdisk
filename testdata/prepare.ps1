function Test-Administrator {
    $user = [Security.Principal.WindowsIdentity]::GetCurrent();
    (New-Object Security.Principal.WindowsPrincipal $user).IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
}

$cwd = Get-Location
$is_admin = Test-Administrator
$data_dir = "$cwd\testdata"
[IO.FileInfo] $data_file = "$data_dir\data.zip"

Write-Output $PSVersionTable
Write-Output "------"
Write-Output "  CWD: $cwd"
Write-Output "Admin: $is_admin"
Write-Output " Data: $data_file, Exists: $($data_file.Exists)"

Expand-Archive $data_file.FullName -DestinationPath $data_dir -Force
Get-ChildItem -Path $data_dir | Out-String