$env:WORK_DIR=(get-location)
$env:LIBCLANG_PATH="${env:WORK_DIR}\LLVM-11.0.0\"
$env:PATH+=";${env:WORK_DIR}\ispc\bin\;${env:LIBCLANG_PATH}"

Write-Output "Building ispc-rs"
cargo build
if (!$?) {
    exit 1
}

Write-Output "Running ispc-rs Tests"
cargo test
if (!$?) {
    exit 1
}

# build the examples
cd examples
Get-ChildItem .\ -Directory | ForEach-Object {
	cd $_
	Write-Output $_
    $dirname = $_ | Split-Path -Leaf
    if ($dirname -eq "simple") {
        cargo build --features ispc
    }

	cargo build
	if (!$?) {
		exit 1
	}
	cd ..
}

