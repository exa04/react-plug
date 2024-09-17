Set-Location Z:\Plugins\react_plug\react_plug_ts\;
bun run build;
Set-Location Z:\Plugins\react_plug\example\gui;
Remove-Item -LiteralPath ".\node_modules" -Force -Recurse;
bun.exe install;
bun.exe dev