{
  description = "Tau webradio server - Nix flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        craneLib = crane.mkLib pkgs;
      in
      {
        packages.default = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;

          nativeBuildInputs = with pkgs; [
            pkg-config
            perl
            rustPlatform.bindgenHook
            openssl
          ];

          OPENSSL_NO_VENDOR = 1;

          doCheck = false;
        };

        # Boot a VM with the module and fetch the broadcast index.
        checks = pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
          nixos-module = pkgs.testers.runNixOSTest {
            name = "tau-tower-module";

            nodes.machine =
              { pkgs, ... }:
              {
                imports = [ self.nixosModules.default ];

                services.tau-tower = {
                  enable = true;
                  corsAllowList = [ "http://localhost:4000" ];
                  credentialsFile = pkgs.writeText "tau-tower-creds" ''
                    TAU_TOWER_USERNAME=testuser
                    TAU_TOWER_PASSWORD=testpass
                  '';
                };

                environment.systemPackages = [ pkgs.curl ];
              };

            testScript = ''
              machine.wait_for_unit("tau-tower.service")
              machine.wait_for_open_port(6001)
              machine.wait_for_open_port(6000)
              # The broadcast index is served immediately; the /<endpoint> mount
              # blocks until a source connects, so we only fetch "/".
              machine.succeed("curl -sf http://127.0.0.1:6001/ >/dev/null")
            '';
          };
        };
      }
    )
    // {
      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          cfg = config.services.tau-tower;

          toml = pkgs.formats.toml { };

          # Render tower.toml from the typed options (snake_case keys; the
          # README's hyphenated spellings would not deserialize). Credentials are
          # left empty: tau-tower overlays them from the environment, keeping this
          # file secret-free and store-safe.
          towerToml = toml.generate "tower.toml" (
            {
              username = "";
              password = "";
              listen_port = cfg.listenPort;
              broadcast_port = cfg.broadcastPort;
              broadcast_endpoint = cfg.broadcastEndpoint;
            }
            // lib.optionalAttrs (cfg.corsAllowList != [ ]) {
              cors_allow_list = cfg.corsAllowList;
            }
          );

          # tau-tower looks for tau/tower.toml under $XDG_CONFIG_HOME (set below).
          configDir = pkgs.runCommand "tau-tower-config" { } ''
            install -Dm444 ${towerToml} "$out/tau/tower.toml"
          '';
        in
        {
          options.services.tau-tower = {
            enable = lib.mkEnableOption "tau-tower webradio broadcast server";

            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
              defaultText = lib.literalExpression "tau-tower.packages.\${pkgs.stdenv.hostPlatform.system}.default";
              description = "The tau-tower package to run; must expose `bin/tau-tower`.";
            };

            listenPort = lib.mkOption {
              type = lib.types.port;
              default = 6000;
              description = ''
                Port for the inbound source stream; the tau-radio client connects
                here over WebSocket. Bound on all interfaces, so keep it behind a
                reverse proxy or firewall.
              '';
            };

            broadcastPort = lib.mkOption {
              type = lib.types.port;
              default = 6001;
              description = "Port serving the outbound Ogg/Opus broadcast (`GET /<broadcastEndpoint>`).";
            };

            broadcastEndpoint = lib.mkOption {
              type = lib.types.str;
              default = "tau.ogg";
              description = ''
                Mount path for the broadcast; "tau.ogg" serves `GET /tau.ogg`.
              '';
            };

            corsAllowList = lib.mkOption {
              type = lib.types.listOf lib.types.str;
              default = [ ];
              example = [ "https://asciinema.example.com" ];
              description = ''
                Origins allowed to fetch the broadcast cross-origin (`[ "*" ]`
                for any). Empty emits no CORS headers, which is fine when the
                player fetches the stream same-origin through the proxy.
              '';
            };

            credentialsFile = lib.mkOption {
              type = lib.types.path;
              example = "/run/secrets/tau-tower.env";
              description = ''
                Environment file (kept out of the Nix store) with the shared
                source credentials, read by tau-tower at start-up:

                ```
                TAU_TOWER_USERNAME=...
                TAU_TOWER_PASSWORD=...
                ```

                They must match the tau-radio client. Provide via sops-nix/agenix
                or a root-owned 0400 file.
              '';
            };

            openFirewall = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = ''
                Open `listenPort` and `broadcastPort` in the firewall. Usually
                left off, since a reverse proxy fronts the service.
              '';
            };
          };

          config = lib.mkIf cfg.enable {
            networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [
              cfg.listenPort
              cfg.broadcastPort
            ];

            systemd.services.tau-tower = {
              description = "tau-tower webradio broadcast server";
              wantedBy = [ "multi-user.target" ];
              after = [ "network.target" ];

              environment.XDG_CONFIG_HOME = configDir;

              serviceConfig = {
                Type = "simple";
                DynamicUser = true;

                # Secrets come from here; tau-tower reads them from the environment.
                EnvironmentFile = cfg.credentialsFile;

                ExecStart = "${cfg.package}/bin/tau-tower";

                Restart = "on-failure";
                RestartSec = 5;

                # Hardening: tau-tower needs no writable paths (reads config, opens sockets).
                ProtectSystem = "strict";
                ProtectHome = true;
                PrivateTmp = true;
                NoNewPrivileges = true;
                ProtectKernelTunables = true;
                ProtectKernelModules = true;
                ProtectControlGroups = true;
                RestrictAddressFamilies = [
                  "AF_INET"
                  "AF_INET6"
                  "AF_UNIX"
                ];
                RestrictNamespaces = true;
                LockPersonality = true;
              };
            };
          };
        };
    };
}
