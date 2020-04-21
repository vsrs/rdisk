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

$partitions = Get-Partition | Select-Object -Property DiskNumber, PartitionNumber, Offset, Size, AccessPaths, Guid, GptType, `
    Type, MbrType, DriveLetter, IsBoot, IsSystem
$partitions | ConvertTo-Json | Out-File -FilePath "$data_dir\partitions.json" -Encoding ascii

$volumes = Get-Volume | Select-Object -Property Path, Size, SizeRemaining, FileSystemType, FileSystemLabel, DriveLetter
$volumes | ConvertTo-Json | Out-File -FilePath "$data_dir\volumes.json" -Encoding ascii

Expand-Archive $data_file.FullName -DestinationPath $data_dir -Force
Get-ChildItem -Path $data_dir | Out-String
