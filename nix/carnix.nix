{ fetchFromGitHub, pkgs, ... }:
let
  fork = fetchFromGitHub {
    owner = "tazjin";
    repo = "carnix";
    rev = "767f129068cbf0121820e19bde7acce360ce38e8";
    sha256 = "0gq3dacp3q41z00a7azwhy0jwrj27jqckb7lfan5p353dbdn861j";
  };

  carnix = import "${fork}" { inherit pkgs; };
in carnix
