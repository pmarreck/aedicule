{
	description = "Generic GPUI frontplane for WAT applications";

	inputs = {
		nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
		fenix = {
			url = "github:nix-community/fenix";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs = { self, nixpkgs, fenix }:
		let
			systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
			forAllSystems = nixpkgs.lib.genAttrs systems;
			pkgsFor = system: import nixpkgs { inherit system; };
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
					frontplaneSource = builtins.path {
						path = ./.;
						name = "gpui-wasm-frontplane-source";
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
									"/src"
									"/third_party"
									"/web"
								]
								|| pkgs.lib.hasPrefix "/src/" relative
								|| pkgs.lib.hasPrefix "/third_party/" relative
								|| pkgs.lib.hasPrefix "/web/" relative;
					};
					testSource = builtins.path {
						path = ./.;
						name = "gpui-wasm-test-source";
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
									"/src"
									"/third_party"
									"/tests"
									"/web"
								]
								|| pkgs.lib.hasPrefix "/src/" relative
								|| pkgs.lib.hasPrefix "/third_party/" relative
								|| pkgs.lib.hasPrefix "/tests/" relative
								|| pkgs.lib.hasPrefix "/web/" relative;
					};
					applicationCargoDeps = pkgs.rustPlatform.fetchCargoVendor {
						name = "aedicule-cargo-deps";
						src = frontplaneSource;
						hash = "sha256-A3f3MI5eTK2QzvEsRHYqav06oXlFGg1exH9T+pM4Iks=";
					};
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
				in {
					frontplane = pkgs.rustPlatform.buildRustPackage {
						pname = "gpui-wasm";
						version = "0.1.0";
						src = frontplaneSource;
						cargoDeps = applicationCargoDeps;
						doCheck = false;
						nativeBuildInputs = with pkgs; [ pkg-config cmake clang ]
							++ pkgs.lib.optionals linux [ makeWrapper ];
						buildInputs = pkgs.lib.optionals linux (linuxLibraries pkgs);
						postFixup = pkgs.lib.optionalString linux ''
							wrapProgram $out/bin/gpui-wasm \
								--prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath (linuxLibraries pkgs)}
							'';
						meta.mainProgram = "gpui-wasm";
					};
					test = pkgs.rustPlatform.buildRustPackage {
						pname = "gpui-wasm-tests";
						version = "0.1.0";
						src = testSource;
						cargoDeps = applicationCargoDeps;
						doCheck = true;
						nativeBuildInputs = with pkgs; [ pkg-config cmake clang nix ripgrep ];
						buildInputs = pkgs.lib.optionals linux (linuxLibraries pkgs);
						checkPhase = ''
							runHook preCheck
							patchShebangs tests/cli/development_dependencies tests/cli/repository_boundary
							cargo test --no-default-features --features native-runtime
							cargo test --bin gpui-wasm
							cargo rustc --release --bin gpui-wasm -- -D warnings
							cargo test --test gui_cli
							cargo check
							./tests/cli/development_dependencies
							./tests/cli/repository_boundary
							runHook postCheck
						'';
						installPhase = ''
							mkdir -p $out
						touch $out/passed
						'';
					};
					webRuntime = (webRustPlatformFor system).buildRustPackage {
						pname = "aedicule-web-runtime";
						version = "0.1.0";
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
							cp web/index.html web/bootstrap.js $out/
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
							ripgrep
						]
							++ pkgs.lib.optionals pkgs.stdenv.isLinux (linuxLibraries pkgs);
						LD_LIBRARY_PATH = pkgs.lib.optionalString pkgs.stdenv.isLinux
							(pkgs.lib.makeLibraryPath (linuxLibraries pkgs));
					};
					web = pkgs.mkShell {
						packages = [ webToolchain ];
					};
				});
		};
}
