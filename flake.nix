{
	description = "Generic GPUI frontplane for WAT applications";

	inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

	outputs = { self, nixpkgs }:
		let
			systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
			forAllSystems = nixpkgs.lib.genAttrs systems;
			pkgsFor = system: import nixpkgs { inherit system; };
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
									"/Cargo.toml"
									"/Cargo.lock"
									"/src"
									"/third_party"
								]
								|| pkgs.lib.hasPrefix "/src/" relative
								|| pkgs.lib.hasPrefix "/third_party/" relative;
					};
				in rec {
					frontplane = pkgs.rustPlatform.buildRustPackage {
						pname = "gpui-wasm";
						version = "0.1.0";
						src = frontplaneSource;
						cargoHash = "sha256-oc/C4bf4JMZkeZQ+n1UgmlIRbW4Tmr446d4BazcSFJM=";
						doCheck = false;
						nativeBuildInputs = with pkgs; [ pkg-config cmake clang ]
							++ pkgs.lib.optionals linux [ makeWrapper ];
						buildInputs = pkgs.lib.optionals linux (linuxLibraries pkgs);
						postFixup = pkgs.lib.optionalString linux ''
							wrapProgram $out/bin/gpui-wasm \
								--prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath (linuxLibraries pkgs)}
							'';
					};
					vibesteroids = pkgs.runCommand "vibesteroids-wat" {} ''
						mkdir -p $out/share/gpui-wasm/plugins
						cp ${./plugins/vibesteroids.wat} $out/share/gpui-wasm/plugins/vibesteroids.wat
					'';
					default = pkgs.runCommand "gpui-wasm-0.1.0" {
						nativeBuildInputs = [ pkgs.makeWrapper ];
					} ''
						mkdir -p $out/bin $out/share/gpui-wasm/plugins
						ln -s ${vibesteroids}/share/gpui-wasm/plugins/vibesteroids.wat \
							$out/share/gpui-wasm/plugins/vibesteroids.wat
						makeWrapper ${frontplane}/bin/gpui-wasm $out/bin/gpui-wasm \
							--set GPUI_WASM_DEFAULT_PLUGIN $out/share/gpui-wasm/plugins/vibesteroids.wat
						makeWrapper ${frontplane}/bin/gpui-wasm-render $out/bin/gpui-wasm-render \
							--set GPUI_WASM_DEFAULT_PLUGIN $out/share/gpui-wasm/plugins/vibesteroids.wat
					'';
				});

			checks = forAllSystems (system: {
				inherit (self.packages.${system}) default;
			});

			devShells = forAllSystems (system:
				let
					pkgs = pkgsFor system;
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
							wasmtime
						]
							++ pkgs.lib.optionals pkgs.stdenv.isLinux (linuxLibraries pkgs);
						LD_LIBRARY_PATH = pkgs.lib.optionalString pkgs.stdenv.isLinux
							(pkgs.lib.makeLibraryPath (linuxLibraries pkgs));
					};
				});
		};
}
