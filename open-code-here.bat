@REM VBWR B
@REM
@REM Project: UlanziDock VibeWare version
@REM Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
@REM Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
@REM
@REM VibeWare: Human intent, AI execution, and plenty of tokens
@REM Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
@REM
@REM Modified with AI: OpenAI Codex; added this header on 2026-10-10.
@REM Human guidance: Umberto Giacobbi; requested VibeWare branding.
@REM
@REM Copyright (c) 2026 Umberto Giacobbi
@REM License: Existing AGPL notice retained below.
@REM
@REM VBWR E

REM YAi!
REM
REM Copyright © 2019-2026 UmbertoGiacobbiDotBiz. All rights reserved.
REM Website: https://umbertogiacobbi.biz
REM Email: hello@umbertogiacobbi.biz
REM
REM This file is part of YAi!.
REM
REM YAi! is free software: you can redistribute it and/or modify it under the terms
REM of the GNU Affero General Public License version 3 as published by the Free
REM Software Foundation.
REM
REM YAi! is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
REM without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
REM PURPOSE. See the GNU Affero General Public License for more details.
REM
REM You should have received a copy of the GNU Affero General Public License along
REM with YAi!. If not, see <https://www.gnu.org/licenses/>.
REM
REM YAi!
REM VS Code workspace launcher

@echo off
setlocal

pushd "%~dp0" >nul || exit /b 1

where code >nul 2>&1
if errorlevel 1 (
	echo VS Code command-line launcher was not found on PATH.
	echo Install the "code" command in PATH, then run this file again.
	popd >nul
	exit /b 1
)

start "" code .

popd >nul
exit /b 0