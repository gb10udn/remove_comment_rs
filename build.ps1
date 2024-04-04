Param(
    [int]$onlyPyBuild
)

Function CreatePythonExe() {
    $hasPyProEnv = Test-Path ".venv_pro"
    if ($hasPyProEnv -eq $false) {
        python -m venv .venv_pro
    }

    .\.venv_pro\Scripts\activate
    python -m pip install --upgrade pip
    pip install -r .\requirements_pro.txt
    pyinstaller vba.py --onefile

    # TODO: 240326 生成に使ったファイルをまとめて削除してもいいかも？
}


Function CreateRsExe() {
    cargo build --release
}


Function CollectFiles() {
    Copy-Item .\target\release\remove_comment_rs.exe .\dist\remove_comment_rs.exe
    Copy-Item config.json .\dist\config.json
    Copy-Item sample_run.cmd .\dist\sample_run.cmd
}


# [START] main process
if (($onlyPyBuild -eq 0) -Or ($onlyPyBuild -eq 1)) {
    CreatePythonExe
    if ($onlyPyBuild -eq 0) {
        CreateRsExe
        CollectFiles
    }
    Write-Host "\n\nFinished"

} else {
    throw "ArgError: onlyPyBuild must be 0 or 1, not $onlyPyBuild"
}
# [END] main process