{
  description = "newsfrwdr";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crate2nix = {
      #url = "github:kolloch/crate2nix";
      # if you use git dependencies with branches in Cargo.toml, use this fork
      # https://github.com/kolloch/crate2nix/issues/205
      #url = "github:yusdacra/crate2nix/feat/builtinfetchgit";
      url = "github:balsoft/crate2nix/balsoft/fix-broken-ifd";
      flake = false;
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, rust-overlay, crate2nix, ... }:
    let
      # If you change the name here, you must also do it in Cargo.toml
      name = "newsfrwdr";
      rustChannel = "stable";
    in
    utils.lib.eachDefaultSystem
      (system:
        let
          # Imports
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlay
              (self: super: {
                # Because rust-overlay bundles multiple rust packages into one
                # derivation, specify that mega-bundle here, so that crate2nix
                # will use them automatically.
                rustc = self.rust-bin.${rustChannel}.latest.default;
                cargo = self.rust-bin.${rustChannel}.latest.default;
              })
            ];
          };
          inherit (import "${crate2nix}/tools.nix" { inherit pkgs; })
            generatedCargoNix;

          # Create the cargo2nix project
          project = pkgs.callPackage
            (generatedCargoNix {
              inherit name;
              src = ./.;
            })
            {
              # Individual crate overrides go here
              # Example: https://github.com/balsoft/simple-osd-daemons/blob/6f85144934c0c1382c7a4d3a2bbb80106776e270/flake.nix#L28-L50
              defaultCrateOverrides = pkgs.defaultCrateOverrides // {
                # The himalaya crate itself is overriden here. Typically we
                # configure non-Rust dependencies (see below) here.
                ${name} = oldAttrs: {
                  inherit buildInputs nativeBuildInputs;
                };
              };
            };

          # Configuration for the non-Rust dependencies
          buildInputs = with pkgs; [ openssl.dev ];
          nativeBuildInputs = with pkgs; [ rustc cargo pkgconfig ];
        in
        rec {
          packages.${name} = project.rootCrate.build;

          # `nix build`
          defaultPackage = packages.${name};

          # `nix run`
          apps.${name} = utils.lib.mkApp {
            inherit name;
            drv = packages.${name};
          };
          defaultApp = apps.${name};

          # `nix develop`
          devShell = pkgs.mkShell
            {
              inputsFrom = builtins.attrValues self.packages.${system};
              buildInputs = buildInputs ++ (with pkgs;
                # Tools you need for development go here.
                [
                  cargo-watch
                  pkgs.rust-bin.${rustChannel}.latest.rust-analysis
                  pkgs.rust-bin.${rustChannel}.latest.rls
                ]);
              RUST_SRC_PATH = "${pkgs.rust-bin.${rustChannel}.latest.rust-src}/lib/rustlib/src/rust/library";
            };
        }
      ) // {
      nixosModule = { pkgs, config, ... }:
        let
          inherit (nixpkgs) lib;
          cfg = config.services.newsfrwdr;
        in
        {
          options.services.newsfrwdr = {
            enable = lib.mkEnableOption "newsfrwdr service";

            config = lib.mkOption {
              type = with lib.types; str;
              default = "";
              example = ''
                [inputs.rdk31]
                url = "https://rdk31.com/atom.xml"

                [[outputs.default]]
                type = "discord_webhook"
                url = "https://discord.com/api/webhooks/abcd..."
              '';
              description = ''
                Config.
                Warning: this is stored in cleartext in the Nix store!
                Use <option>configFile</option> instead.
              '';
            };

            configFile = lib.mkOption {
              type = with lib.types; nullOr path;
              default = null;
              example = "/run/keys/newsfrwdr-config";
              description = ''
                Config file.
              '';
            };
          };

          config =
            let
              configFile = if (cfg.configFile != null) then cfg.configFile else
              pkgs.writeText "newsfrwdr-config.toml" cfg.config;
            in
            lib.mkIf cfg.enable {
              nixpkgs.overlays = [ self.overlay ];

              systemd.services.newsfrwdr = {
                description = "newsfrwdr";
                after = [ "network.target" ];
                wantedBy = [ "multi-user.target" ];
                serviceConfig.ExecStart = "${pkgs.newsfrwdr}/bin/newsfrwdr -c ${configFile}";
              };
            };
        };

      overlay = final: prev: {
        ${name} = self.defaultPackage.${prev.system};
      };
    };
}
