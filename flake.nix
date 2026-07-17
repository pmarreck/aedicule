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
									"/build.rs"
									"/Cargo.toml"
									"/Cargo.lock"
									"/WAT_ABI.md"
									"/src"
									"/third_party"
								]
								|| pkgs.lib.hasPrefix "/src/" relative
								|| pkgs.lib.hasPrefix "/third_party/" relative;
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
								]
								|| pkgs.lib.hasPrefix "/src/" relative
								|| pkgs.lib.hasPrefix "/third_party/" relative
								|| pkgs.lib.hasPrefix "/tests/" relative;
					};
				in {
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
						meta.mainProgram = "gpui-wasm";
					};
					test = pkgs.rustPlatform.buildRustPackage {
						pname = "gpui-wasm-tests";
						version = "0.1.0";
						src = testSource;
						cargoHash = "sha256-oc/C4bf4JMZkeZQ+n1UgmlIRbW4Tmr446d4BazcSFJM=";
						doCheck = true;
						nativeBuildInputs = with pkgs; [ pkg-config cmake clang nix ripgrep ];
						buildInputs = pkgs.lib.optionals linux (linuxLibraries pkgs);
						checkPhase = ''
							runHook preCheck
							patchShebangs tests/cli/development_dependencies tests/cli/repository_boundary
							cargo test --no-default-features
							cargo test --bin gpui-wasm
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
					default = self.packages.${system}.frontplane;
				});

			checks = forAllSystems (system: {
				frontplane = self.packages.${system}.frontplane;
				test = self.packages.${system}.test;
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
						]
							++ pkgs.lib.optionals pkgs.stdenv.isLinux (linuxLibraries pkgs);
						LD_LIBRARY_PATH = pkgs.lib.optionalString pkgs.stdenv.isLinux
							(pkgs.lib.makeLibraryPath (linuxLibraries pkgs));
					};
				});
		};
}
