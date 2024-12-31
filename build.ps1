$ErrorActionPreference = "Stop"

$targets = @{
    "x86_64-unknown-linux-gnu" = "silex-lsp-linux"
    "aarch64-unknown-linux-gnu" = "silex-lsp-linux-arm64"
    "x86_64-pc-windows-gnu" = "silex-lsp-win.exe"
}

foreach ($target in $targets.Keys) {
    $outputName = $targets[$target]
    Write-Host "Building for target: $target -> $outputName"
    cross build --release --target $target

    # Ensure the server directory exists
    if (-not (Test-Path -Path "server")) {
        New-Item -ItemType Directory -Path "server"
    }

    # Copy the binary to the server directory with the desired name
    $sourcePath = "target/$target/release/silex-lsp"
    $destPath = "server/$outputName"
    if (Test-Path "$sourcePath") {
        Copy-Item "$sourcePath" "$destPath"
    } else {
        $sourcePath = "target/$target/release/silex-lsp.exe"
        if (Test-Path "$sourcePath") {
            Copy-Item "$sourcePath" "$destPath"
        } else {
            throw "Binary not found for target: $target"
        }
    }

    Write-Host "Binary placed at $destPath"
}
