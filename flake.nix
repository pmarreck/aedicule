{
	description = "Generic GPUI frontplane for WAT applications";

	inputs = {
		nixpkgs.url = "github:pmarreck/nixpkgs/150cb35d9ab619bcc7f0476c9018000fd1a47c27";
		fenix = {
			url = "github:nix-community/fenix";
			inputs.nixpkgs.follows = "nixpkgs";
		};
		crane.url = "github:ipetkov/crane";
	};

	outputs = { self, nixpkgs, fenix, crane }:
		let
			version = "0.1.1";
			systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
			forAllSystems = nixpkgs.lib.genAttrs systems;
			pkgsFor = system: import nixpkgs {
				inherit system;
				config.allowUnsupportedSystem = true;
			};
			luajitWithPackagesFor = pkgs: pkgs.luajit.withPackages (ps: with ps; [
				# Enable only modules a project tool actually imports. Common choices:
				# alt-getopt
				# basexx
				# busted
				# cjson
				# lpeg
				# lua_cliargs
				# luabitop
				# luacheck
				# luafilesystem
				# luasocket
				# luasystem
				# moonscript
				# penlight
				# sqlite
				# tl
			]);
			webToolchainFor = system:
				let
					fenixPackages = fenix.packages.${system};
				in fenixPackages.combine [
					fenixPackages.minimal.cargo
					fenixPackages.minimal.rustc
					fenixPackages.complete.rust-src
					fenixPackages.targets."wasm32-unknown-unknown".latest.rust-std
				];
			webRustPlatformFor = system:
				let toolchain = webToolchainFor system;
				in (pkgsFor system).makeRustPlatform {
					cargo = toolchain;
					rustc = toolchain;
				};
			nativeCrossToolchainFor = system: rustTarget:
				let fenixPackages = fenix.packages.${system};
				in fenixPackages.combine [
					fenixPackages.minimal.cargo
					fenixPackages.minimal.rustc
					fenixPackages.targets.${rustTarget}.latest.rust-std
				];
			wasmBindgenArtifacts = {
				x86_64-linux = {
					target = "x86_64-unknown-linux-musl";
					hash = "sha256-BklI1Y4tbAp0UhZHemObppYhbWMJqqkCk50bhlsdhp0=";
				};
				aarch64-linux = {
					target = "aarch64-unknown-linux-musl";
					hash = "sha256-IkUSAlSp9smprfNgHz1SuzEwkhnpzqt2ludOJIhcRAo=";
				};
				aarch64-darwin = {
					target = "aarch64-apple-darwin";
					hash = "sha256-ffU2ur40XetogoFI29xxF5EYr9q0LYNUfHzr+/FCa9U=";
				};
			};
			wasmBindgenFor = system:
				let
					pkgs = pkgsFor system;
					artifact = wasmBindgenArtifacts.${system};
				in pkgs.stdenvNoCC.mkDerivation {
					pname = "wasm-bindgen-cli";
					version = "0.2.126";
					src = pkgs.fetchurl {
						url = "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/0.2.126/wasm-bindgen-0.2.126-${artifact.target}.tar.gz";
						hash = artifact.hash;
					};
					nativeBuildInputs = [ pkgs.gnutar ];
					unpackPhase = "tar -xzf $src";
					installPhase = ''
						install -Dm755 wasm-bindgen-0.2.126-${artifact.target}/wasm-bindgen \
							$out/bin/wasm-bindgen
					'';
				};
			linuxLibraries = pkgs: with pkgs; [
				alsa-lib
				fontconfig
				freetype
				libGL
				libxkbcommon
				vulkan-loader
				wayland
				libx11
				libxcb
				libxcursor
				libxi
				libxrandr
			];
		in {
			packages = forAllSystems (system:
				let
					pkgs = pkgsFor system;
					linux = pkgs.stdenv.isLinux;
					luajitWithPackages = luajitWithPackagesFor pkgs;
					libcrypto = "${pkgs.openssl.out}/lib/libcrypto${pkgs.stdenv.hostPlatform.extensions.sharedLibrary}";
					cargoBuildSourceName = "gpui-wasm-build-source";
					frontplaneSource = builtins.path {
						path = ./.;
						name = cargoBuildSourceName;
						filter = path: type:
							let
								root = toString ./.;
								relative = pkgs.lib.removePrefix root (toString path);
							in relative == ""
								|| builtins.elem relative [
									"/build.rs"
									"/Cargo.toml"
									"/Cargo.lock"
									"/WAT_ABI.md"
									"/assets"
									"/assets/icons"
									"/assets/icons/Aedicule.ico"
									"/packaging"
									"/packaging/windows"
									"/packaging/windows/aedicule.rc"
									"/src"
									"/third_party"
									"/web"
									"/web/cargo-config.toml"
								]
								|| pkgs.lib.hasPrefix "/src/" relative
								|| pkgs.lib.hasPrefix "/third_party/" relative
								;
					};
					testSource = builtins.path {
						path = ./.;
						name = cargoBuildSourceName;
						filter = path: type:
							let
								root = toString ./.;
								relative = pkgs.lib.removePrefix root (toString path);
							in relative == ""
								|| relative == "/.github"
								|| builtins.elem relative [
									"/build.rs"
									"/Cargo.toml"
									"/Cargo.lock"
									"/README.md"
									"/WAT_ABI.md"
									"/build"
									"/build_all"
									"/check_reproducible"
									"/delivery-targets"
									"/demos"
									"/flake.lock"
									"/flake.nix"
									"/assets"
									"/packaging"
									"/src"
									"/test"
									"/third_party"
									"/tests"
									"/web"
								]
								|| pkgs.lib.hasPrefix "/.github/" relative
								|| pkgs.lib.hasPrefix "/assets/" relative
								|| pkgs.lib.hasPrefix "/demos/" relative
								|| pkgs.lib.hasPrefix "/packaging/" relative
								|| pkgs.lib.hasPrefix "/src/" relative
								|| pkgs.lib.hasPrefix "/third_party/" relative
								|| pkgs.lib.hasPrefix "/tests/" relative
								|| pkgs.lib.hasPrefix "/web/" relative;
					};
					nativeCrane = crane.mkLib pkgs;
					cargoDependencySource = nativeCrane.mkDummySrc {
						src = frontplaneSource;
						# These two local compatibility crates are dependencies, not
						# application source. Preserve their implementations while Crane
						# replaces ordinary crate targets with stable dummy sources.
						extraDummyScript = ''
							chmod u+w $out/third_party/ztracing/src/lib.rs
							chmod u+w $out/third_party/ztracing_macro/src/lib.rs
							cp -R ${./third_party/ztracing/src}/. $out/third_party/ztracing/src/
							cp -R ${./third_party/ztracing_macro/src}/. \
								$out/third_party/ztracing_macro/src/
						'';
					};
					applicationCargoDeps = pkgs.rustPlatform.fetchCargoVendor {
						name = "aedicule-cargo-deps";
						src = cargoDependencySource;
						hash = "sha256-Q8ruCyX5lQTlpWMyHBnu0Y5mDzBILwBjl3mzbwFZiVE=";
					};
					nativeCommonArgs = {
						pname = "gpui-wasm";
						inherit version;
						src = frontplaneSource;
						dummySrc = cargoDependencySource;
						# Keep nixpkgs' fixed-output Cargo vendor layout while Crane splits
						# dependency compilation from source compilation. The dummy source
						# makes this dependency derivation stable across Rust-only edits.
						cargoVendorDir = null;
						cargoDeps = applicationCargoDeps;
						cargoExtraArgs = "--bin gpui-wasm --bin gpui-wasm-render";
						strictDeps = true;
						nativeBuildInputs = with pkgs; [
							rustPlatform.cargoSetupHook
							pkg-config
							cmake
							clang
						];
						buildInputs = pkgs.lib.optionals linux (linuxLibraries pkgs);
					};
					nativeCargoArtifacts = nativeCrane.buildDepsOnly (
						(builtins.removeAttrs nativeCommonArgs [ "src" ]) // {
						# The package and warning gates need release artifacts; test-only
						# dependencies are a smaller delta compiled by the test derivation.
						doCheck = false;
						buildPhaseCargoCommand =
							"cargoWithProfile build --bin gpui-wasm --bin gpui-wasm-render";
						}
					);
					gpuiWebFont = path: hash: pkgs.fetchurl {
						url = "https://raw.githubusercontent.com/pmarreck/zed/7e34550622005f62cd337d465cb5fd25c2ce8bd7/assets/fonts/${path}";
						inherit hash;
					};
					gpuiWebFonts = {
						plexRegular = gpuiWebFont "ibm-plex-sans/IBMPlexSans-Regular.ttf" "sha256-l13No32A8Djc0UPCLjPKLZegzFqSmqzhx0kVOw/hr6U=";
						plexItalic = gpuiWebFont "ibm-plex-sans/IBMPlexSans-Italic.ttf" "sha256-qcbvmULEnknRHhGm2swLOgh5eHV+myKga4rCKmQA+xU=";
						plexSemiBold = gpuiWebFont "ibm-plex-sans/IBMPlexSans-SemiBold.ttf" "sha256-ogyvgoYCOmp6heQLHSpK6fw+Ox+e2o9MVC3UmGr2e7E=";
						plexSemiBoldItalic = gpuiWebFont "ibm-plex-sans/IBMPlexSans-SemiBoldItalic.ttf" "sha256-FHEGthlCOoK0D21kus4HStW3huWfSAI72mnWOxdV+4I=";
						lilexRegular = gpuiWebFont "lilex/Lilex-Regular.ttf" "sha256-ikZ61aGUHcfJzrruNOPtk2SnO0g5MDTjY+S1KL1xHQQ=";
						lilexBold = gpuiWebFont "lilex/Lilex-Bold.ttf" "sha256-M2khTi7N20WuUCSxHUXdY36Z6dISercpgX1B1yuKnGg=";
						lilexItalic = gpuiWebFont "lilex/Lilex-Italic.ttf" "sha256-puUgRrke0Spq9Qu3TnDifepKVOqyvtiUfbTBEouoPUU=";
						lilexBoldItalic = gpuiWebFont "lilex/Lilex-BoldItalic.ttf" "sha256-lim9Ze5zdoeUFFYONygWU8qC3PM+FD10czQbL4ggaYU=";
					};
					# `-Z build-std` resolves through Rust's library lockfile as well as
					# Aedicule's. GPUI web also embeds fonts outside its crate root, so
					# the sandbox reconstructs those pinned repository-relative assets.
					webCargoDeps = pkgs.runCommand "aedicule-web-cargo-deps" {} ''
						mkdir -p $out
						cp -R ${applicationCargoDeps}/. $out/
						chmod -R u+w $out
						cp -R ${webToolchainFor system}/lib/rustlib/src/rust/library/vendor/. \
							$out/source-registry-0/
						install -Dm444 ${gpuiWebFonts.plexRegular} $out/assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf
						install -Dm444 ${gpuiWebFonts.plexItalic} $out/assets/fonts/ibm-plex-sans/IBMPlexSans-Italic.ttf
						install -Dm444 ${gpuiWebFonts.plexSemiBold} $out/assets/fonts/ibm-plex-sans/IBMPlexSans-SemiBold.ttf
						install -Dm444 ${gpuiWebFonts.plexSemiBoldItalic} $out/assets/fonts/ibm-plex-sans/IBMPlexSans-SemiBoldItalic.ttf
						install -Dm444 ${gpuiWebFonts.lilexRegular} $out/assets/fonts/lilex/Lilex-Regular.ttf
						install -Dm444 ${gpuiWebFonts.lilexBold} $out/assets/fonts/lilex/Lilex-Bold.ttf
						install -Dm444 ${gpuiWebFonts.lilexItalic} $out/assets/fonts/lilex/Lilex-Italic.ttf
						install -Dm444 ${gpuiWebFonts.lilexBoldItalic} $out/assets/fonts/lilex/Lilex-BoldItalic.ttf
					'';
					macosRustTarget = "aarch64-apple-darwin";
					macosToolchain = nativeCrossToolchainFor system macosRustTarget;
					macosRustPlatform = pkgs.makeRustPlatform {
						cargo = macosToolchain;
						rustc = macosToolchain;
					};
					fetchMacosSdk = pkgs.callPackage
						(nixpkgs + "/pkgs/by-name/ap/apple-sdk/common/fetch-sdk.nix") {};
					macosSdk = fetchMacosSdk {
						urls = [
							"https://swcdn.apple.com/content/downloads/14/48/052-59890-A_I0F5YGAY0Y/p9n40hio7892gou31o1v031ng6fnm9sb3c/CLTools_macOSNMOS_SDK.pkg"
							"https://web.archive.org/web/20250211001355/https://swcdn.apple.com/content/downloads/14/48/052-59890-A_I0F5YGAY0Y/p9n40hio7892gou31o1v031ng6fnm9sb3c/CLTools_macOSNMOS_SDK.pkg"
						];
						version = "14.4";
						hash = "sha256-QozDiwY0Czc0g45vPD7G4v4Ra+3DujCJbSads3fJjjM=";
					};
					linuxAarch64Pkgs = if system == "aarch64-linux"
						then pkgs
						else pkgs.pkgsCross."aarch64-multiplatform";
					linuxX86_64Pkgs = if system == "x86_64-linux"
						then pkgs
						else pkgs.pkgsCross.gnu64;
					windowsAarch64Pkgs = pkgs.pkgsCross."mingw-ucrt-aarch64";
					windowsX86_64Pkgs = pkgs.pkgsCross."mingw-ucrt-x86_64";
					mkRawFrontplane = targetName: targetPkgs:
						let
							targetLinux = targetPkgs.stdenv.hostPlatform.isLinux;
							targetLlvmWindows = targetPkgs.stdenv.hostPlatform.isWindows
								&& (targetPkgs.stdenv.hostPlatform.useLLVM or false);
						in targetPkgs.rustPlatform.buildRustPackage {
							pname = "aedicule-${targetName}";
							inherit version;
							src = frontplaneSource;
							cargoDeps = applicationCargoDeps;
							cargoBuildFlags = [ "--bin" "gpui-wasm" ];
							cargoInstallFlags = [ "--bin" "gpui-wasm" ];
							doCheck = false;
							strictDeps = true;
							nativeBuildInputs = with targetPkgs.buildPackages; [
								pkg-config
								cmake
								clang
							];
							# embed-resource 3.0.11 misidentifies LLVM's GNU-compatible
							# `llvm-windres` banner when Nix exports RC. Let it infer the
							# target-prefixed windres executable, which selects GNU mode.
							preBuild = targetPkgs.lib.optionalString targetLlvmWindows ''
								unset RC
							'';
							# MinGW's COFF symbol ordering varies between otherwise identical
							# links. Removing the release symbol table also normalizes the PE
							# checksum, yielding byte-reproducible Windows executables.
							stripDebugFlags = if targetPkgs.stdenv.hostPlatform.isWindows
								then [ "--strip-all" ]
								else [ "-S" "-p" ];
							buildInputs = targetPkgs.lib.optionals targetLinux
								(linuxLibraries targetPkgs);
						};
					mkLinuxDelivery = targetName: targetPkgs: raw:
						let
							dynamicLinker = targetPkgs.stdenv.cc.bintools.dynamicLinker;
							loaderName = builtins.baseNameOf dynamicLinker;
							runtimeClosure = pkgs.closureInfo {
								rootPaths = [ raw ] ++ (linuxLibraries targetPkgs) ++ [
									targetPkgs.stdenv.cc.cc.lib
									targetPkgs.stdenv.cc.libc.out
								];
							};
						in pkgs.runCommand "aedicule-delivery-${targetName}" {
							nativeBuildInputs = [ pkgs.patchelf ];
						} ''
							mkdir -p $out/bin $out/lib $out/libexec $out/share/aedicule/web
							cp -R ${self.packages.${system}.webRuntime}/. $out/share/aedicule/web/
							while IFS= read -r storePath; do
								for libraryDirectory in "$storePath/lib" "$storePath/lib64"; do
									if [[ -d "$libraryDirectory" ]]; then
										while IFS= read -r -d $'\0' library; do
											name=$(basename "$library")
											if [[ ! -e "$out/lib/$name" ]]; then
												cp -L "$library" "$out/lib/$name"
											fi
										done < <(find "$libraryDirectory" -maxdepth 1 \
											\( -type f -o -type l \) -name '*.so*' -print0)
									fi
								done
							done < ${runtimeClosure}/store-paths
							install -Dm755 ${dynamicLinker} $out/lib/${loaderName}
							if [[ -d ${targetPkgs.alsa-lib}/lib/alsa-lib ]]; then
								cp -LR ${targetPkgs.alsa-lib}/lib/alsa-lib $out/lib/
							fi
							mkdir -p $out/share/X11
							cp -LR ${targetPkgs.alsa-lib}/share/alsa $out/share/
							cp -LR ${targetPkgs.xkeyboard_config}/share/X11/. $out/share/X11/
							cp -LR ${targetPkgs.libx11}/share/X11/locale $out/share/X11/
							install -Dm755 ${raw}/bin/gpui-wasm $out/libexec/aedicule
							chmod -R u+w $out/lib $out/libexec
							install -Dm755 ${./packaging/linux/aedicule-launcher.in} $out/bin/aedicule
							substituteInPlace $out/bin/aedicule \
								--replace-fail '@LOADER@' '${loaderName}'
							patchelf --set-interpreter /aedicule/launch-through-bin-aedicule \
								--set-rpath '$ORIGIN/../lib' $out/libexec/aedicule
							while IFS= read -r -d $'\0' candidate; do
								if rpath=$(patchelf --print-rpath "$candidate" 2>/dev/null) && \
									[[ "$rpath" == *'/nix/store/'* ]]; then
									patchelf --set-rpath '$ORIGIN' "$candidate"
								fi
							done < <(find $out/lib -type f -print0)
							install -Dm644 ${./packaging/linux/aedicule.desktop} \
								$out/share/applications/aedicule.desktop
							install -Dm644 ${./packaging/linux/org.webassembly.wat.xml} \
								$out/share/mime/packages/org.webassembly.wat.xml
							install -Dm644 ${./assets/icons/aedicule-app.png} \
								$out/share/icons/hicolor/512x512/apps/aedicule.png
							install -Dm644 ${./assets/icons/aedicule-wat-document.png} \
								$out/share/icons/hicolor/512x512/mimetypes/org.webassembly.wat.png
							install -Dm644 ${./demos/ulam-flower.wat} \
								$out/share/aedicule/demos/ulam-flower.wat
							install -Dm644 ${./demos/vibesteroids.wat} \
								$out/share/aedicule/demos/vibesteroids.wat
							install -Dm644 ${./demos/manifest.tsv} \
								$out/share/aedicule/demos/manifest.tsv
							${pkgs.lib.optionalString (system == "x86_64-linux" && targetName == "linux-x86_64") ''
								env -i HOME=$TMPDIR PATH=${pkgs.bash}/bin:${pkgs.coreutils}/bin \
									${pkgs.bash}/bin/bash $out/bin/aedicule --help >/dev/null
							''}
						'';
					mkWindowsDelivery = targetName: raw:
						pkgs.runCommand "aedicule-delivery-${targetName}" {} ''
							mkdir -p $out/Demos $out/Tools $out/Web
							cp -R ${self.packages.${system}.webRuntime}/. $out/Web/
							cp ${raw}/bin/gpui-wasm.exe $out/Aedicule.exe
							while IFS= read -r -d $'\0' runtimeDll; do
								cp -L "$runtimeDll" $out/
							done < <(find ${raw}/bin -maxdepth 1 \
								\( -type f -o -type l \) -iname '*.dll' -print0)
							cp ${./assets/icons/Aedicule.ico} $out/Aedicule.ico
							cp ${./assets/icons/Aedicule-WAT.ico} $out/Aedicule-WAT.ico
							cp ${./packaging/windows/install-file-association.ps1} \
								$out/Tools/install-file-association.ps1
							cp ${./demos/ulam-flower.wat} $out/Demos/ulam-flower.wat
							cp ${./demos/vibesteroids.wat} $out/Demos/vibesteroids.wat
							cp ${./demos/manifest.tsv} $out/Demos/manifest.tsv
						'';
					mkMacosDelivery = raw:
						pkgs.runCommand "aedicule-delivery-macos-aarch64" {
							nativeBuildInputs = [ pkgs.imagemagick pkgs.libicns ];
						} ''
							app=$out/Aedicule.app
							mkdir -p $app/Contents/MacOS $app/Contents/Resources/Demos $app/Contents/Resources/Web icon-work/app icon-work/document
							cp -R ${self.packages.${system}.webRuntime}/. $app/Contents/Resources/Web/
							cp ${raw}/bin/gpui-wasm $app/Contents/MacOS/aedicule
							chmod +x $app/Contents/MacOS/aedicule
							for size in 16 32 48 128 256 512 1024; do
								magick ${./assets/icons/aedicule-app.png} -resize "''${size}x''${size}" \
									"icon-work/app/''${size}.png"
								magick ${./assets/icons/aedicule-wat-document.png} -resize "''${size}x''${size}" \
									"icon-work/document/''${size}.png"
							done
							png2icns $app/Contents/Resources/Aedicule.icns icon-work/app/*.png
							png2icns $app/Contents/Resources/AediculeWAT.icns icon-work/document/*.png
							cp ${./packaging/macos/Info.plist.in} $app/Contents/Info.plist
							substituteInPlace $app/Contents/Info.plist \
								--replace-fail '@VERSION@' '${version}'
							cp ${./demos/ulam-flower.wat} $app/Contents/Resources/Demos/ulam-flower.wat
							cp ${./demos/vibesteroids.wat} $app/Contents/Resources/Demos/vibesteroids.wat
							cp ${./demos/manifest.tsv} $app/Contents/Resources/Demos/manifest.tsv
						'';
					archiveName = targetName: extension:
						"aedicule-${version}-${targetName}.${extension}";
					mkZipRelease = targetName: rootName: delivery:
						pkgs.runCommand "aedicule-release-${targetName}" {
							nativeBuildInputs = [ pkgs.coreutils pkgs.findutils pkgs.zip ];
						} ''
							mkdir -p package/${rootName} $out
							cp -R ${delivery}/. package/${rootName}/
							find package -exec touch -h -d @1 {} +
							(
								cd package
								find ${rootName} -print | LC_ALL=C sort | \
									zip -X -q ${archiveName targetName "zip"} -@
							)
							mv package/${archiveName targetName "zip"} $out/
						'';
					mkTarRelease = targetName: rootName: delivery:
						pkgs.runCommand "aedicule-release-${targetName}" {
							nativeBuildInputs = [ pkgs.coreutils pkgs.findutils pkgs.gnutar pkgs.gzip ];
						} ''
							mkdir -p package/${rootName} $out
							cp -R ${delivery}/. package/${rootName}/
							tar --sort=name --mtime=@1 --owner=0 --group=0 --numeric-owner \
								-C package -czf $out/${archiveName targetName "tar.gz"} ${rootName}
						'';
					rawMacosAarch64 = macosRustPlatform.buildRustPackage {
						pname = "aedicule-macos-aarch64";
						inherit version;
						src = frontplaneSource;
						cargoDeps = applicationCargoDeps;
						doCheck = false;
						nativeBuildInputs = [
							pkgs.cargo-zigbuild
							pkgs.cmake
							pkgs.clang
							pkgs.llvmPackages.libclang
							luajitWithPackages
							pkgs.openssl
							pkgs.rcodesign
						];
						LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
						AEDICULE_LIBCRYPTO = libcrypto;
						SDKROOT = macosSdk;
						MACOSX_DEPLOYMENT_TARGET = "11.0";
						buildPhase = ''
							runHook preBuild
							export CARGO_ZIGBUILD_CACHE_DIR=$TMPDIR/cargo-zigbuild
							export ZIG_GLOBAL_CACHE_DIR=$TMPDIR/zig-cache
							cargo zigbuild --offline --release --target ${macosRustTarget} \
								--bin gpui-wasm
							binary=target/${macosRustTarget}/release/gpui-wasm
							luajit ${./packaging/macos/canonicalize_uuid} "$binary"
							rcodesign sign --binary-identifier com.pmarreck.aedicule \
								"$binary" "$binary.signed"
							mv "$binary.signed" "$binary"
							runHook postBuild
						'';
						installPhase = ''
							install -Dm755 target/${macosRustTarget}/release/gpui-wasm \
								$out/bin/gpui-wasm
						'';
					};
					rawLinuxAarch64 = mkRawFrontplane "linux-aarch64" linuxAarch64Pkgs;
					rawLinuxX86_64 = mkRawFrontplane "linux-x86_64" linuxX86_64Pkgs;
					rawWindowsAarch64 = mkRawFrontplane "windows-aarch64" windowsAarch64Pkgs;
					rawWindowsX86_64 = mkRawFrontplane "windows-x86_64" windowsX86_64Pkgs;
					releaseWeb = mkZipRelease "web" "Aedicule-web"
						self.packages.${system}.delivery-web;
					releaseMacosAarch64 = mkZipRelease "macos-aarch64" "Aedicule"
						self.packages.${system}.delivery-macos-aarch64;
					releaseLinuxAarch64 = mkTarRelease "linux-aarch64" "Aedicule"
						self.packages.${system}.delivery-linux-aarch64;
					releaseWindowsAarch64 = mkZipRelease "windows-aarch64" "Aedicule"
						self.packages.${system}.delivery-windows-aarch64;
					releaseLinuxX86_64 = mkTarRelease "linux-x86_64" "Aedicule"
						self.packages.${system}.delivery-linux-x86_64;
					releaseWindowsX86_64 = mkZipRelease "windows-x86_64" "Aedicule"
						self.packages.${system}.delivery-windows-x86_64;
				in {
					cargo-artifacts-native = nativeCargoArtifacts;
					frontplane = nativeCrane.buildPackage (nativeCommonArgs // {
						cargoArtifacts = nativeCargoArtifacts;
						doCheck = false;
						nativeBuildInputs = nativeCommonArgs.nativeBuildInputs
							++ pkgs.lib.optionals linux [ pkgs.makeWrapper ];
						postFixup = pkgs.lib.optionalString linux ''
							wrapProgram $out/bin/gpui-wasm \
								--prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath (linuxLibraries pkgs)} \
								--set AEDICULE_WEB_RUNTIME ${self.packages.${system}.webRuntime}
							'';
						meta.mainProgram = "gpui-wasm";
					});
					test = nativeCrane.buildPackage (nativeCommonArgs // {
						pname = "gpui-wasm-tests";
						src = testSource;
						cargoArtifacts = nativeCargoArtifacts;
						doCheck = true;
						nativeBuildInputs = nativeCommonArgs.nativeBuildInputs ++ [
							pkgs.actionlint
							pkgs.pkg-config
							pkgs.cmake
							pkgs.clang
							pkgs.nix
							pkgs.nodejs
							pkgs.ripgrep
							luajitWithPackages
							pkgs.openssl
						];
						AEDICULE_LIBCRYPTO = libcrypto;
						buildInputs = pkgs.lib.optionals linux (linuxLibraries pkgs);
						checkPhase = ''
							runHook preCheck
							patchShebangs tests/cli/development_dependencies tests/cli/repository_boundary \
								tests/cli/document_packages tests/cli/demo_snapshots \
								tests/cli/macos_reproducibility tests/cli/reproducible_releases \
								tests/cli/web_i18n tests/cli/github_pages \
								tests/cli/ci_acceleration \
								tests/cli/native_parallelism \
								packaging/macos/canonicalize_uuid check_reproducible
							cargo test --no-default-features --features native-runtime
							cargo test --bin gpui-wasm
							cargo rustc --release --bin gpui-wasm -- -D warnings
							cargo test --test gui_cli
							cargo check
							./tests/cli/development_dependencies
							./tests/cli/repository_boundary
							./tests/cli/document_packages
							./tests/cli/demo_snapshots
							./tests/cli/macos_reproducibility
							./tests/cli/reproducible_releases
							./tests/cli/web_i18n
							./tests/cli/github_pages
							./tests/cli/ci_acceleration
							# Daemon-dependent flake mutation and real package realization
							# remain in the outer ./test gate; this sandbox runs pure checks.
							./tests/cli/native_parallelism
							node ./tests/integration/web_browser_startup_unit.mjs
							runHook postCheck
						'';
						installPhase = ''
							mkdir -p $out
							touch $out/passed
						'';
					});
					webRuntime = (webRustPlatformFor system).buildRustPackage {
						pname = "aedicule-web-runtime";
						inherit version;
						src = frontplaneSource;
						cargoDeps = webCargoDeps;
						doCheck = false;
						nativeBuildInputs = [ (wasmBindgenFor system) pkgs.binaryen ];
						buildPhase = ''
							runHook preBuild
							cargo --config web/cargo-config.toml build --release \
								--target wasm32-unknown-unknown --no-default-features \
								--features web --bin aedicule-web
							wasm-bindgen --target web --out-name aedicule_web --out-dir bindgen \
								target/wasm32-unknown-unknown/release/aedicule-web.wasm
							wasm-opt --enable-threads -Oz bindgen/aedicule_web_bg.wasm \
								-o bindgen/aedicule_web_bg.optimized.wasm
							mv bindgen/aedicule_web_bg.optimized.wasm bindgen/aedicule_web_bg.wasm
							runHook postBuild
						'';
						installPhase = ''
							mkdir -p $out
							install -Dm644 ${./web/index.html} $out/index.html
							install -Dm644 ${./web/bootstrap.js} $out/bootstrap.js
							install -Dm644 ${./web/coi-serviceworker.js} $out/coi-serviceworker.js
							install -Dm644 ${./packaging/web/manifest.webmanifest} $out/manifest.webmanifest
							install -Dm644 ${./assets/icons/aedicule-app.png} $out/icon.png
							cp bindgen/aedicule_web.js bindgen/aedicule_web_bg.wasm $out/
						'';
					};
					webFallback = self.lib.${system}.webBundle {
						wat = ./src/fallback.wat;
					};
					webServe = pkgs.writeShellApplication {
						name = "aedicule-web-serve";
						runtimeInputs = [ pkgs.caddy ];
						text = ''
							if (( $# != 1 )); then
								printf 'usage: %s DIRECTORY\n' "$0" >&2
								exit 2
							fi
							AEDICULE_WEB_ROOT="$(cd "$1" && pwd)"
							export AEDICULE_WEB_ROOT
							exec caddy run --config ${./web/Caddyfile} --adapter caddyfile
						'';
					};
					frontplane-macos-aarch64 = rawMacosAarch64;
					frontplane-linux-aarch64 = rawLinuxAarch64;
					frontplane-linux-x86_64 = rawLinuxX86_64;
					frontplane-windows-aarch64 = rawWindowsAarch64;
					frontplane-windows-x86_64 = rawWindowsX86_64;
					delivery-macos-aarch64 = mkMacosDelivery rawMacosAarch64;
					delivery-linux-aarch64 = mkLinuxDelivery "linux-aarch64" linuxAarch64Pkgs rawLinuxAarch64;
					delivery-linux-x86_64 = mkLinuxDelivery "linux-x86_64" linuxX86_64Pkgs rawLinuxX86_64;
					delivery-windows-aarch64 = mkWindowsDelivery "windows-aarch64" rawWindowsAarch64;
					delivery-windows-x86_64 = mkWindowsDelivery "windows-x86_64" rawWindowsX86_64;
					delivery-web =
						let
							ulam = self.lib.${system}.webBundle {
								wat = ./demos/ulam-flower.wat;
								title = "Ulam Flower — Aedicule";
							};
							vibesteroids = self.lib.${system}.webBundle {
								wat = ./demos/vibesteroids.wat;
								title = "Vibesteroids — Aedicule";
							};
						in pkgs.runCommand "aedicule-delivery-web" {} ''
							mkdir -p $out/ulam-flower $out/vibesteroids
							cp ${./packaging/web/index.html} $out/index.html
							cp ${./packaging/web/launcher.mjs} $out/launcher.mjs
							cp ${./packaging/web/launcher-i18n.mjs} $out/launcher-i18n.mjs
							cp ${./web/coi-serviceworker.js} $out/coi-serviceworker.js
							cp ${./packaging/web/manifest.webmanifest} $out/manifest.webmanifest
							cp ${./assets/icons/aedicule-app.png} $out/icon.png
							cp -R ${ulam}/. $out/ulam-flower/
							cp -R ${vibesteroids}/. $out/vibesteroids/
							cp ${./demos/manifest.tsv} $out/manifest.tsv
						'';
					delivery-all = pkgs.runCommand "aedicule-delivery-all" {} ''
						mkdir -p $out
						ln -s ${self.packages.${system}.delivery-web} $out/web
						ln -s ${self.packages.${system}.delivery-macos-aarch64} $out/macos-aarch64
						ln -s ${self.packages.${system}.delivery-linux-aarch64} $out/linux-aarch64
						ln -s ${self.packages.${system}.delivery-windows-aarch64} $out/windows-aarch64
						ln -s ${self.packages.${system}.delivery-linux-x86_64} $out/linux-x86_64
						ln -s ${self.packages.${system}.delivery-windows-x86_64} $out/windows-x86_64
					'';
					release-web = releaseWeb;
					release-macos-aarch64 = releaseMacosAarch64;
					release-linux-aarch64 = releaseLinuxAarch64;
					release-windows-aarch64 = releaseWindowsAarch64;
					release-linux-x86_64 = releaseLinuxX86_64;
					release-windows-x86_64 = releaseWindowsX86_64;
					release-all = pkgs.runCommand "aedicule-release-all" {
						nativeBuildInputs = [ pkgs.coreutils ];
					} ''
						mkdir -p $out
						cp ${releaseWeb}/${archiveName "web" "zip"} $out/
						cp ${releaseMacosAarch64}/${archiveName "macos-aarch64" "zip"} $out/
						cp ${releaseLinuxAarch64}/${archiveName "linux-aarch64" "tar.gz"} $out/
						cp ${releaseWindowsAarch64}/${archiveName "windows-aarch64" "zip"} $out/
						cp ${releaseLinuxX86_64}/${archiveName "linux-x86_64" "tar.gz"} $out/
						cp ${releaseWindowsX86_64}/${archiveName "windows-x86_64" "zip"} $out/
						cp ${./delivery-targets} $out/delivery-targets
						cp ${./demos/manifest.tsv} $out/demo-manifest.tsv
						(cd $out && sha256sum aedicule-* > SHA256SUMS)
					'';
					default = self.packages.${system}.frontplane;
				});

			checks = forAllSystems (system: {
				frontplane = self.packages.${system}.frontplane;
				test = self.packages.${system}.test;
				webRuntime = self.packages.${system}.webRuntime;
				webFallback = self.packages.${system}.webFallback;
			});

			lib = forAllSystems (system:
				let pkgs = pkgsFor system;
				in {
					webBundle = { wat, title ? "Aedicule web application" }:
						pkgs.runCommand "aedicule-web-bundle" {} ''
							mkdir -p $out
							cp -R ${self.packages.${system}.webRuntime}/. $out/
							cp ${wat} $out/code.wat
							substituteInPlace $out/index.html \
								--replace-fail 'Aedicule web application' ${pkgs.lib.escapeShellArg title}
						'';
				});

			devShells = forAllSystems (system:
				let
					pkgs = pkgsFor system;
					webToolchain = webToolchainFor system;
					luajitWithPackages = luajitWithPackagesFor pkgs;
				in {
					default = pkgs.mkShell {
						packages = with pkgs; [
							rustc
							cargo
							rustfmt
							clippy
							pkg-config
							cmake
							clang
							cargo-nextest
							actionlint
							nix
							nodejs
							ripgrep
							luajitWithPackages
							openssl
						]
							++ pkgs.lib.optionals pkgs.stdenv.isLinux (linuxLibraries pkgs);
						LD_LIBRARY_PATH = pkgs.lib.optionalString pkgs.stdenv.isLinux
							(pkgs.lib.makeLibraryPath (linuxLibraries pkgs));
						AEDICULE_LIBCRYPTO = "${pkgs.openssl.out}/lib/libcrypto${pkgs.stdenv.hostPlatform.extensions.sharedLibrary}";
					};
					web = pkgs.mkShell {
						packages = [ webToolchain ];
					};
				});
		};
}
