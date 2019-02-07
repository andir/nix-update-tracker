# Generated by carnix 0.9.2: carnix generate-nix --src .
{ lib, buildPlatform, buildRustCrate, buildRustCrateHelpers, cratesIO, fetchgit }:
with buildRustCrateHelpers;
let inherit (lib.lists) fold;
    inherit (lib.attrsets) recursiveUpdate;
in
let crates = cratesIO // rec {
# nix-vuln-scanner-0.1.2-dev

  crates.nix_vuln_scanner."0.1.2-dev" = deps: { features?(features_.nix_vuln_scanner."0.1.2-dev" deps {}) }: buildRustCrate {
    crateName = "nix-vuln-scanner";
    version = "0.1.2-dev";
    authors = [ "Andreas Rammhold <andreas@rammhold.de>" ];
    src = exclude [ ".git" "target" ] ./.;
    dependencies = mapFeatures features ([
      (cratesIO.crates."actix_web"."${deps."nix_vuln_scanner"."0.1.2-dev"."actix_web"}" deps)
      (cratesIO.crates."brotli"."${deps."nix_vuln_scanner"."0.1.2-dev"."brotli"}" deps)
      (cratesIO.crates."chrono"."${deps."nix_vuln_scanner"."0.1.2-dev"."chrono"}" deps)
      (cratesIO.crates."clap"."${deps."nix_vuln_scanner"."0.1.2-dev"."clap"}" deps)
      (cratesIO.crates."diesel"."${deps."nix_vuln_scanner"."0.1.2-dev"."diesel"}" deps)
      (cratesIO.crates."error_chain"."${deps."nix_vuln_scanner"."0.1.2-dev"."error_chain"}" deps)
      (cratesIO.crates."fern"."${deps."nix_vuln_scanner"."0.1.2-dev"."fern"}" deps)
      (cratesIO.crates."fs2"."${deps."nix_vuln_scanner"."0.1.2-dev"."fs2"}" deps)
      (cratesIO.crates."itertools"."${deps."nix_vuln_scanner"."0.1.2-dev"."itertools"}" deps)
      (cratesIO.crates."lazy_static"."${deps."nix_vuln_scanner"."0.1.2-dev"."lazy_static"}" deps)
      (cratesIO.crates."libflate"."${deps."nix_vuln_scanner"."0.1.2-dev"."libflate"}" deps)
      (cratesIO.crates."log"."${deps."nix_vuln_scanner"."0.1.2-dev"."log"}" deps)
      (cratesIO.crates."md5"."${deps."nix_vuln_scanner"."0.1.2-dev"."md5"}" deps)
      (cratesIO.crates."mktemp"."${deps."nix_vuln_scanner"."0.1.2-dev"."mktemp"}" deps)
      (cratesIO.crates."num_cpus"."${deps."nix_vuln_scanner"."0.1.2-dev"."num_cpus"}" deps)
      (cratesIO.crates."rayon"."${deps."nix_vuln_scanner"."0.1.2-dev"."rayon"}" deps)
      (cratesIO.crates."regex"."${deps."nix_vuln_scanner"."0.1.2-dev"."regex"}" deps)
      (cratesIO.crates."reqwest"."${deps."nix_vuln_scanner"."0.1.2-dev"."reqwest"}" deps)
      (cratesIO.crates."serde"."${deps."nix_vuln_scanner"."0.1.2-dev"."serde"}" deps)
      (cratesIO.crates."serde_derive"."${deps."nix_vuln_scanner"."0.1.2-dev"."serde_derive"}" deps)
      (cratesIO.crates."serde_json"."${deps."nix_vuln_scanner"."0.1.2-dev"."serde_json"}" deps)
      (cratesIO.crates."serde_yaml"."${deps."nix_vuln_scanner"."0.1.2-dev"."serde_yaml"}" deps)
      (cratesIO.crates."threadpool"."${deps."nix_vuln_scanner"."0.1.2-dev"."threadpool"}" deps)
      (cratesIO.crates."time"."${deps."nix_vuln_scanner"."0.1.2-dev"."time"}" deps)
      (cratesIO.crates."walkdir"."${deps."nix_vuln_scanner"."0.1.2-dev"."walkdir"}" deps)
    ]);
  };
  features_.nix_vuln_scanner."0.1.2-dev" = deps: f: updateFeatures f (rec {
    actix_web."${deps.nix_vuln_scanner."0.1.2-dev".actix_web}".default = true;
    brotli."${deps.nix_vuln_scanner."0.1.2-dev".brotli}".default = (f.brotli."${deps.nix_vuln_scanner."0.1.2-dev".brotli}".default or false);
    chrono."${deps.nix_vuln_scanner."0.1.2-dev".chrono}".default = true;
    clap."${deps.nix_vuln_scanner."0.1.2-dev".clap}".default = true;
    diesel = fold recursiveUpdate {} [
      { "${deps.nix_vuln_scanner."0.1.2-dev".diesel}"."sqlite" = true; }
      { "${deps.nix_vuln_scanner."0.1.2-dev".diesel}".default = true; }
    ];
    error_chain."${deps.nix_vuln_scanner."0.1.2-dev".error_chain}".default = true;
    fern."${deps.nix_vuln_scanner."0.1.2-dev".fern}".default = true;
    fs2."${deps.nix_vuln_scanner."0.1.2-dev".fs2}".default = true;
    itertools."${deps.nix_vuln_scanner."0.1.2-dev".itertools}".default = true;
    lazy_static."${deps.nix_vuln_scanner."0.1.2-dev".lazy_static}".default = true;
    libflate."${deps.nix_vuln_scanner."0.1.2-dev".libflate}".default = true;
    log."${deps.nix_vuln_scanner."0.1.2-dev".log}".default = true;
    md5."${deps.nix_vuln_scanner."0.1.2-dev".md5}".default = true;
    mktemp."${deps.nix_vuln_scanner."0.1.2-dev".mktemp}".default = true;
    nix_vuln_scanner."0.1.2-dev".default = (f.nix_vuln_scanner."0.1.2-dev".default or true);
    num_cpus."${deps.nix_vuln_scanner."0.1.2-dev".num_cpus}".default = true;
    rayon."${deps.nix_vuln_scanner."0.1.2-dev".rayon}".default = true;
    regex."${deps.nix_vuln_scanner."0.1.2-dev".regex}".default = true;
    reqwest."${deps.nix_vuln_scanner."0.1.2-dev".reqwest}".default = true;
    serde."${deps.nix_vuln_scanner."0.1.2-dev".serde}".default = true;
    serde_derive."${deps.nix_vuln_scanner."0.1.2-dev".serde_derive}".default = true;
    serde_json."${deps.nix_vuln_scanner."0.1.2-dev".serde_json}".default = true;
    serde_yaml."${deps.nix_vuln_scanner."0.1.2-dev".serde_yaml}".default = true;
    threadpool."${deps.nix_vuln_scanner."0.1.2-dev".threadpool}".default = true;
    time."${deps.nix_vuln_scanner."0.1.2-dev".time}".default = true;
    walkdir."${deps.nix_vuln_scanner."0.1.2-dev".walkdir}".default = true;
  }) [
    (cratesIO.features_.actix_web."${deps."nix_vuln_scanner"."0.1.2-dev"."actix_web"}" deps)
    (cratesIO.features_.brotli."${deps."nix_vuln_scanner"."0.1.2-dev"."brotli"}" deps)
    (cratesIO.features_.chrono."${deps."nix_vuln_scanner"."0.1.2-dev"."chrono"}" deps)
    (cratesIO.features_.clap."${deps."nix_vuln_scanner"."0.1.2-dev"."clap"}" deps)
    (cratesIO.features_.diesel."${deps."nix_vuln_scanner"."0.1.2-dev"."diesel"}" deps)
    (cratesIO.features_.error_chain."${deps."nix_vuln_scanner"."0.1.2-dev"."error_chain"}" deps)
    (cratesIO.features_.fern."${deps."nix_vuln_scanner"."0.1.2-dev"."fern"}" deps)
    (cratesIO.features_.fs2."${deps."nix_vuln_scanner"."0.1.2-dev"."fs2"}" deps)
    (cratesIO.features_.itertools."${deps."nix_vuln_scanner"."0.1.2-dev"."itertools"}" deps)
    (cratesIO.features_.lazy_static."${deps."nix_vuln_scanner"."0.1.2-dev"."lazy_static"}" deps)
    (cratesIO.features_.libflate."${deps."nix_vuln_scanner"."0.1.2-dev"."libflate"}" deps)
    (cratesIO.features_.log."${deps."nix_vuln_scanner"."0.1.2-dev"."log"}" deps)
    (cratesIO.features_.md5."${deps."nix_vuln_scanner"."0.1.2-dev"."md5"}" deps)
    (cratesIO.features_.mktemp."${deps."nix_vuln_scanner"."0.1.2-dev"."mktemp"}" deps)
    (cratesIO.features_.num_cpus."${deps."nix_vuln_scanner"."0.1.2-dev"."num_cpus"}" deps)
    (cratesIO.features_.rayon."${deps."nix_vuln_scanner"."0.1.2-dev"."rayon"}" deps)
    (cratesIO.features_.regex."${deps."nix_vuln_scanner"."0.1.2-dev"."regex"}" deps)
    (cratesIO.features_.reqwest."${deps."nix_vuln_scanner"."0.1.2-dev"."reqwest"}" deps)
    (cratesIO.features_.serde."${deps."nix_vuln_scanner"."0.1.2-dev"."serde"}" deps)
    (cratesIO.features_.serde_derive."${deps."nix_vuln_scanner"."0.1.2-dev"."serde_derive"}" deps)
    (cratesIO.features_.serde_json."${deps."nix_vuln_scanner"."0.1.2-dev"."serde_json"}" deps)
    (cratesIO.features_.serde_yaml."${deps."nix_vuln_scanner"."0.1.2-dev"."serde_yaml"}" deps)
    (cratesIO.features_.threadpool."${deps."nix_vuln_scanner"."0.1.2-dev"."threadpool"}" deps)
    (cratesIO.features_.time."${deps."nix_vuln_scanner"."0.1.2-dev"."time"}" deps)
    (cratesIO.features_.walkdir."${deps."nix_vuln_scanner"."0.1.2-dev"."walkdir"}" deps)
  ];


# end

}; in

rec {
  nix_vuln_scanner = crates.crates.nix_vuln_scanner."0.1.2-dev" deps;
  __all = [ (nix_vuln_scanner {}) ];
  deps.actix."0.7.9" = {
    actix_derive = "0.3.2";
    bitflags = "1.0.4";
    bytes = "0.4.11";
    crossbeam_channel = "0.3.8";
    failure = "0.1.5";
    fnv = "1.0.6";
    futures = "0.1.25";
    log = "0.4.6";
    parking_lot = "0.7.1";
    smallvec = "0.6.8";
    tokio = "0.1.15";
    tokio_codec = "0.1.1";
    tokio_executor = "0.1.6";
    tokio_io = "0.1.11";
    tokio_reactor = "0.1.8";
    tokio_signal = "0.2.7";
    tokio_tcp = "0.1.3";
    tokio_timer = "0.2.10";
    trust_dns_proto = "0.5.0";
    trust_dns_resolver = "0.10.3";
    uuid = "0.7.2";
    libc = "0.2.48";
  };
  deps.actix_net."0.2.6" = {
    actix = "0.7.9";
    bytes = "0.4.11";
    futures = "0.1.25";
    log = "0.4.6";
    mio = "0.6.16";
    net2 = "0.2.33";
    num_cpus = "1.9.0";
    slab = "0.4.2";
    tokio = "0.1.15";
    tokio_codec = "0.1.1";
    tokio_current_thread = "0.1.4";
    tokio_io = "0.1.11";
    tokio_reactor = "0.1.8";
    tokio_tcp = "0.1.3";
    tokio_timer = "0.2.10";
    tower_service = "0.1.0";
    trust_dns_resolver = "0.10.3";
  };
  deps.actix_web."0.7.18" = {
    actix = "0.7.9";
    actix_net = "0.2.6";
    base64 = "0.10.1";
    bitflags = "1.0.4";
    brotli2 = "0.3.2";
    byteorder = "1.3.1";
    bytes = "0.4.11";
    cookie = "0.11.0";
    encoding = "0.2.33";
    failure = "0.1.5";
    flate2 = "1.0.6";
    futures = "0.1.25";
    futures_cpupool = "0.1.8";
    h2 = "0.1.16";
    http = "0.1.15";
    httparse = "1.3.3";
    language_tags = "0.2.2";
    lazy_static = "1.2.0";
    lazycell = "1.2.1";
    log = "0.4.6";
    mime = "0.3.13";
    mime_guess = "2.0.0";
    mio = "0.6.16";
    net2 = "0.2.33";
    num_cpus = "1.9.0";
    parking_lot = "0.7.1";
    percent_encoding = "1.0.1";
    rand = "0.6.5";
    regex = "1.1.0";
    serde = "1.0.87";
    serde_json = "1.0.38";
    serde_urlencoded = "0.5.4";
    sha1 = "0.6.0";
    slab = "0.4.2";
    smallvec = "0.6.8";
    time = "0.1.42";
    tokio = "0.1.15";
    tokio_current_thread = "0.1.4";
    tokio_io = "0.1.11";
    tokio_reactor = "0.1.8";
    tokio_tcp = "0.1.3";
    tokio_timer = "0.2.10";
    url = "1.7.2";
    v_htmlescape = "0.3.2";
    version_check = "0.1.5";
  };
  deps.actix_derive."0.3.2" = {
    proc_macro2 = "0.4.27";
    quote = "0.6.11";
    syn = "0.15.26";
  };
  deps.adler32."1.0.3" = {};
  deps.aho_corasick."0.6.9" = {
    memchr = "2.1.3";
  };
  deps.alloc_no_stdlib."1.3.0" = {};
  deps.ansi_term."0.11.0" = {
    winapi = "0.3.6";
  };
  deps.arc_swap."0.3.7" = {};
  deps.arrayvec."0.4.10" = {
    nodrop = "0.1.13";
  };
  deps.atty."0.2.11" = {
    termion = "1.5.1";
    libc = "0.2.48";
    winapi = "0.3.6";
  };
  deps.autocfg."0.1.2" = {};
  deps.backtrace."0.3.13" = {
    cfg_if = "0.1.6";
    rustc_demangle = "0.1.13";
    autocfg = "0.1.2";
    backtrace_sys = "0.1.28";
    libc = "0.2.48";
    winapi = "0.3.6";
  };
  deps.backtrace_sys."0.1.28" = {
    libc = "0.2.48";
    cc = "1.0.29";
  };
  deps.base64."0.9.3" = {
    byteorder = "1.3.1";
    safemem = "0.3.0";
  };
  deps.base64."0.10.1" = {
    byteorder = "1.3.1";
  };
  deps.bitflags."0.9.1" = {};
  deps.bitflags."1.0.4" = {};
  deps.brotli."2.5.1" = {
    alloc_no_stdlib = "1.3.0";
    brotli_decompressor = "1.3.1";
  };
  deps.brotli_decompressor."1.3.1" = {
    alloc_no_stdlib = "1.3.0";
  };
  deps.brotli_sys."0.3.2" = {
    libc = "0.2.48";
    cc = "1.0.29";
  };
  deps.brotli2."0.3.2" = {
    brotli_sys = "0.3.2";
    libc = "0.2.48";
  };
  deps.build_const."0.2.1" = {};
  deps.byteorder."1.3.1" = {};
  deps.bytes."0.4.11" = {
    byteorder = "1.3.1";
    iovec = "0.1.2";
  };
  deps.cc."1.0.29" = {};
  deps.cfg_if."0.1.6" = {};
  deps.chrono."0.4.6" = {
    num_integer = "0.1.39";
    num_traits = "0.2.6";
    time = "0.1.42";
  };
  deps.clap."2.32.0" = {
    atty = "0.2.11";
    bitflags = "1.0.4";
    strsim = "0.7.0";
    textwrap = "0.10.0";
    unicode_width = "0.1.5";
    vec_map = "0.8.1";
    ansi_term = "0.11.0";
  };
  deps.cloudabi."0.0.3" = {
    bitflags = "1.0.4";
  };
  deps.cookie."0.11.0" = {
    base64 = "0.9.3";
    ring = "0.13.5";
    time = "0.1.42";
    url = "1.7.2";
  };
  deps.core_foundation."0.2.3" = {
    core_foundation_sys = "0.2.3";
    libc = "0.2.48";
  };
  deps.core_foundation_sys."0.2.3" = {
    libc = "0.2.48";
  };
  deps.crc."1.8.1" = {
    build_const = "0.2.1";
  };
  deps.crc32fast."1.1.2" = {
    cfg_if = "0.1.6";
  };
  deps.crossbeam."0.6.0" = {
    cfg_if = "0.1.6";
    crossbeam_channel = "0.3.8";
    crossbeam_deque = "0.6.3";
    crossbeam_epoch = "0.7.1";
    crossbeam_utils = "0.6.5";
    lazy_static = "1.2.0";
    num_cpus = "1.9.0";
    parking_lot = "0.7.1";
  };
  deps.crossbeam_channel."0.3.8" = {
    crossbeam_utils = "0.6.5";
    smallvec = "0.6.8";
  };
  deps.crossbeam_deque."0.2.0" = {
    crossbeam_epoch = "0.3.1";
    crossbeam_utils = "0.2.2";
  };
  deps.crossbeam_deque."0.6.3" = {
    crossbeam_epoch = "0.7.1";
    crossbeam_utils = "0.6.5";
  };
  deps.crossbeam_epoch."0.3.1" = {
    arrayvec = "0.4.10";
    cfg_if = "0.1.6";
    crossbeam_utils = "0.2.2";
    lazy_static = "1.2.0";
    memoffset = "0.2.1";
    nodrop = "0.1.13";
    scopeguard = "0.3.3";
  };
  deps.crossbeam_epoch."0.7.1" = {
    arrayvec = "0.4.10";
    cfg_if = "0.1.6";
    crossbeam_utils = "0.6.5";
    lazy_static = "1.2.0";
    memoffset = "0.2.1";
    scopeguard = "0.3.3";
  };
  deps.crossbeam_utils."0.2.2" = {
    cfg_if = "0.1.6";
  };
  deps.crossbeam_utils."0.6.5" = {
    cfg_if = "0.1.6";
    lazy_static = "1.2.0";
  };
  deps.diesel."1.4.1" = {
    byteorder = "1.3.1";
    diesel_derives = "1.4.0";
    libsqlite3_sys = "0.12.0";
  };
  deps.diesel_derives."1.4.0" = {
    proc_macro2 = "0.4.27";
    quote = "0.6.11";
    syn = "0.15.26";
  };
  deps.dtoa."0.4.3" = {};
  deps.either."1.5.0" = {};
  deps.encoding."0.2.33" = {
    encoding_index_japanese = "1.20141219.5";
    encoding_index_korean = "1.20141219.5";
    encoding_index_simpchinese = "1.20141219.5";
    encoding_index_singlebyte = "1.20141219.5";
    encoding_index_tradchinese = "1.20141219.5";
  };
  deps.encoding_index_japanese."1.20141219.5" = {
    encoding_index_tests = "0.1.4";
  };
  deps.encoding_index_korean."1.20141219.5" = {
    encoding_index_tests = "0.1.4";
  };
  deps.encoding_index_simpchinese."1.20141219.5" = {
    encoding_index_tests = "0.1.4";
  };
  deps.encoding_index_singlebyte."1.20141219.5" = {
    encoding_index_tests = "0.1.4";
  };
  deps.encoding_index_tradchinese."1.20141219.5" = {
    encoding_index_tests = "0.1.4";
  };
  deps.encoding_index_tests."0.1.4" = {};
  deps.encoding_rs."0.8.16" = {
    cfg_if = "0.1.6";
  };
  deps.error_chain."0.8.1" = {
    backtrace = "0.3.13";
  };
  deps.error_chain."0.12.0" = {
    backtrace = "0.3.13";
  };
  deps.failure."0.1.5" = {
    backtrace = "0.3.13";
    failure_derive = "0.1.5";
  };
  deps.failure_derive."0.1.5" = {
    proc_macro2 = "0.4.27";
    quote = "0.6.11";
    syn = "0.15.26";
    synstructure = "0.10.1";
  };
  deps.fern."0.5.7" = {
    log = "0.4.6";
  };
  deps.flate2."1.0.6" = {
    crc32fast = "1.1.2";
    libc = "0.2.48";
    miniz_sys = "0.1.11";
    miniz_oxide_c_api = "0.2.1";
  };
  deps.fnv."1.0.6" = {};
  deps.foreign_types."0.3.2" = {
    foreign_types_shared = "0.1.1";
  };
  deps.foreign_types_shared."0.1.1" = {};
  deps.fs2."0.4.3" = {
    libc = "0.2.48";
    winapi = "0.3.6";
  };
  deps.fuchsia_cprng."0.1.1" = {};
  deps.fuchsia_zircon."0.3.3" = {
    bitflags = "1.0.4";
    fuchsia_zircon_sys = "0.3.3";
  };
  deps.fuchsia_zircon_sys."0.3.3" = {};
  deps.futures."0.1.25" = {};
  deps.futures_cpupool."0.1.8" = {
    futures = "0.1.25";
    num_cpus = "1.9.0";
  };
  deps.h2."0.1.16" = {
    byteorder = "1.3.1";
    bytes = "0.4.11";
    fnv = "1.0.6";
    futures = "0.1.25";
    http = "0.1.15";
    indexmap = "1.0.2";
    log = "0.4.6";
    slab = "0.4.2";
    string = "0.1.3";
    tokio_io = "0.1.11";
  };
  deps.hostname."0.1.5" = {
    libc = "0.2.48";
    winutil = "0.1.1";
  };
  deps.http."0.1.15" = {
    bytes = "0.4.11";
    fnv = "1.0.6";
    itoa = "0.4.3";
  };
  deps.httparse."1.3.3" = {};
  deps.hyper."0.11.27" = {
    base64 = "0.9.3";
    bytes = "0.4.11";
    futures = "0.1.25";
    futures_cpupool = "0.1.8";
    httparse = "1.3.3";
    iovec = "0.1.2";
    language_tags = "0.2.2";
    log = "0.4.6";
    mime = "0.3.13";
    net2 = "0.2.33";
    percent_encoding = "1.0.1";
    relay = "0.1.1";
    time = "0.1.42";
    tokio_core = "0.1.17";
    tokio_io = "0.1.11";
    tokio_service = "0.1.0";
    unicase = "2.2.0";
    want = "0.0.4";
  };
  deps.hyper_tls."0.1.4" = {
    futures = "0.1.25";
    hyper = "0.11.27";
    native_tls = "0.1.5";
    tokio_core = "0.1.17";
    tokio_io = "0.1.11";
    tokio_service = "0.1.0";
    tokio_tls = "0.1.4";
  };
  deps.idna."0.1.5" = {
    matches = "0.1.8";
    unicode_bidi = "0.3.4";
    unicode_normalization = "0.1.8";
  };
  deps.indexmap."1.0.2" = {};
  deps.iovec."0.1.2" = {
    libc = "0.2.48";
    winapi = "0.2.8";
  };
  deps.ipconfig."0.1.9" = {
    error_chain = "0.8.1";
    socket2 = "0.3.8";
    widestring = "0.2.2";
    winapi = "0.3.6";
    winreg = "0.5.1";
  };
  deps.itertools."0.7.11" = {
    either = "1.5.0";
  };
  deps.itoa."0.4.3" = {};
  deps.kernel32_sys."0.2.2" = {
    winapi = "0.2.8";
    winapi_build = "0.1.1";
  };
  deps.language_tags."0.2.2" = {};
  deps.lazy_static."0.2.11" = {};
  deps.lazy_static."1.2.0" = {};
  deps.lazycell."1.2.1" = {};
  deps.libc."0.2.48" = {};
  deps.libflate."0.1.19" = {
    adler32 = "1.0.3";
    byteorder = "1.3.1";
    crc32fast = "1.1.2";
  };
  deps.libsqlite3_sys."0.12.0" = {
    pkg_config = "0.3.14";
  };
  deps.linked_hash_map."0.4.2" = {};
  deps.linked_hash_map."0.5.1" = {};
  deps.lock_api."0.1.5" = {
    owning_ref = "0.4.0";
    scopeguard = "0.3.3";
  };
  deps.log."0.4.6" = {
    cfg_if = "0.1.6";
  };
  deps.lru_cache."0.1.1" = {
    linked_hash_map = "0.4.2";
  };
  deps.matches."0.1.8" = {};
  deps.md5."0.3.8" = {};
  deps.memchr."2.1.3" = {
    cfg_if = "0.1.6";
    libc = "0.2.48";
  };
  deps.memoffset."0.2.1" = {};
  deps.mime."0.3.13" = {
    unicase = "2.2.0";
  };
  deps.mime_guess."2.0.0-alpha_6" = {
    mime = "0.3.13-";
    phf = "0.7.24-";
    unicase = "1.4.2-";
    phf_codegen = "0.7.24-";
  };
  deps.miniz_sys."0.1.11" = {
    libc = "0.2.48";
    cc = "1.0.29";
  };
  deps.miniz_oxide."0.2.1" = {
    adler32 = "1.0.3";
  };
  deps.miniz_oxide_c_api."0.2.1" = {
    crc = "1.8.1";
    libc = "0.2.48";
    miniz_oxide = "0.2.1";
    cc = "1.0.29";
  };
  deps.mio."0.6.16" = {
    iovec = "0.1.2";
    lazycell = "1.2.1";
    log = "0.4.6";
    net2 = "0.2.33";
    slab = "0.4.2";
    fuchsia_zircon = "0.3.3";
    fuchsia_zircon_sys = "0.3.3";
    libc = "0.2.48";
    kernel32_sys = "0.2.2";
    miow = "0.2.1";
    winapi = "0.2.8";
  };
  deps.mio_uds."0.6.7" = {
    iovec = "0.1.2";
    libc = "0.2.48";
    mio = "0.6.16";
  };
  deps.miow."0.2.1" = {
    kernel32_sys = "0.2.2";
    net2 = "0.2.33";
    winapi = "0.2.8";
    ws2_32_sys = "0.2.1";
  };
  deps.mktemp."0.3.1" = {
    uuid = "0.1.18";
  };
  deps.native_tls."0.1.5" = {
    lazy_static = "0.2.11";
    libc = "0.2.48";
    security_framework = "0.1.16";
    security_framework_sys = "0.1.16";
    tempdir = "0.3.7";
    openssl = "0.9.24";
    schannel = "0.1.14";
  };
  deps.net2."0.2.33" = {
    cfg_if = "0.1.6";
    libc = "0.2.48";
    winapi = "0.3.6";
  };
  deps.nix_vuln_scanner."0.1.2-dev" = {
    actix_web = "0.7.18";
    brotli = "2.5.1-";
    chrono = "0.4.6-";
    clap = "2.32.0-";
    diesel = "1.4.1-";
    error_chain = "0.12.0-";
    fern = "0.5.7-";
    fs2 = "0.4.3-";
    itertools = "0.7.11-";
    lazy_static = "1.2.0-";
    libflate = "0.1.19-";
    log = "0.4.6-";
    md5 = "0.3.8-";
    mktemp = "0.3.1-";
    num_cpus = "1.9.0-";
    rayon = "1.0.3-";
    regex = "1.1.0-";
    reqwest = "0.8.8-";
    serde = "1.0.87-";
    serde_derive = "1.0.87-";
    serde_json = "1.0.38-";
    serde_yaml = "0.8.8-";
    threadpool = "1.7.1-";
    time = "0.1.42-";
    walkdir = "2.2.7-";
  };
  deps.nodrop."0.1.13" = {};
  deps.nom."4.2.0" = {
    memchr = "2.1.3";
    version_check = "0.1.5";
  };
  deps.num_integer."0.1.39" = {
    num_traits = "0.2.6";
  };
  deps.num_traits."0.2.6" = {};
  deps.num_cpus."1.9.0" = {
    libc = "0.2.48";
  };
  deps.openssl."0.9.24" = {
    bitflags = "0.9.1";
    foreign_types = "0.3.2";
    lazy_static = "1.2.0";
    libc = "0.2.48";
    openssl_sys = "0.9.40";
  };
  deps.openssl_sys."0.9.40" = {
    libc = "0.2.48";
    cc = "1.0.29";
    pkg_config = "0.3.14";
  };
  deps.owning_ref."0.4.0" = {
    stable_deref_trait = "1.1.1";
  };
  deps.parking_lot."0.7.1" = {
    lock_api = "0.1.5";
    parking_lot_core = "0.4.0";
  };
  deps.parking_lot_core."0.4.0" = {
    rand = "0.6.5";
    smallvec = "0.6.8";
    rustc_version = "0.2.3";
    libc = "0.2.48";
    winapi = "0.3.6";
  };
  deps.percent_encoding."1.0.1" = {};
  deps.phf."0.7.24" = {
    phf_shared = "0.7.24";
  };
  deps.phf_codegen."0.7.24" = {
    phf_generator = "0.7.24";
    phf_shared = "0.7.24";
  };
  deps.phf_generator."0.7.24" = {
    phf_shared = "0.7.24";
    rand = "0.6.5";
  };
  deps.phf_shared."0.7.24" = {
    siphasher = "0.2.3";
    unicase = "1.4.2";
  };
  deps.pkg_config."0.3.14" = {};
  deps.proc_macro2."0.4.27" = {
    unicode_xid = "0.1.0";
  };
  deps.quick_error."1.2.2" = {};
  deps.quote."0.6.11" = {
    proc_macro2 = "0.4.27";
  };
  deps.rand."0.3.23" = {
    libc = "0.2.48";
    rand = "0.4.6";
  };
  deps.rand."0.4.6" = {
    rand_core = "0.3.1";
    rdrand = "0.4.0";
    fuchsia_cprng = "0.1.1";
    libc = "0.2.48";
    winapi = "0.3.6";
  };
  deps.rand."0.5.6" = {
    rand_core = "0.3.1";
    cloudabi = "0.0.3";
    fuchsia_cprng = "0.1.1";
    libc = "0.2.48";
    winapi = "0.3.6";
  };
  deps.rand."0.6.5" = {
    rand_chacha = "0.1.1";
    rand_core = "0.4.0";
    rand_hc = "0.1.0";
    rand_isaac = "0.1.1";
    rand_jitter = "0.1.3";
    rand_os = "0.1.2";
    rand_pcg = "0.1.1";
    rand_xorshift = "0.1.1";
    autocfg = "0.1.2";
    libc = "0.2.48";
    winapi = "0.3.6";
  };
  deps.rand_chacha."0.1.1" = {
    rand_core = "0.3.1";
    autocfg = "0.1.2";
  };
  deps.rand_core."0.3.1" = {
    rand_core = "0.4.0";
  };
  deps.rand_core."0.4.0" = {};
  deps.rand_hc."0.1.0" = {
    rand_core = "0.3.1";
  };
  deps.rand_isaac."0.1.1" = {
    rand_core = "0.3.1";
  };
  deps.rand_jitter."0.1.3" = {
    rand_core = "0.4.0";
    libc = "0.2.48";
    winapi = "0.3.6";
  };
  deps.rand_os."0.1.2" = {
    rand_core = "0.4.0";
    rdrand = "0.4.0";
    cloudabi = "0.0.3";
    fuchsia_cprng = "0.1.1";
    libc = "0.2.48";
    winapi = "0.3.6";
  };
  deps.rand_pcg."0.1.1" = {
    rand_core = "0.3.1";
    rustc_version = "0.2.3";
  };
  deps.rand_xorshift."0.1.1" = {
    rand_core = "0.3.1";
  };
  deps.rayon."1.0.3" = {
    crossbeam_deque = "0.2.0";
    either = "1.5.0";
    rayon_core = "1.4.1";
  };
  deps.rayon_core."1.4.1" = {
    crossbeam_deque = "0.2.0";
    lazy_static = "1.2.0";
    libc = "0.2.48";
    num_cpus = "1.9.0";
  };
  deps.rdrand."0.4.0" = {
    rand_core = "0.3.1";
  };
  deps.redox_syscall."0.1.51" = {};
  deps.redox_termios."0.1.1" = {
    redox_syscall = "0.1.51";
  };
  deps.regex."1.1.0" = {
    aho_corasick = "0.6.9";
    memchr = "2.1.3";
    regex_syntax = "0.6.5";
    thread_local = "0.3.6";
    utf8_ranges = "1.0.2";
  };
  deps.regex_syntax."0.6.5" = {
    ucd_util = "0.1.3";
  };
  deps.relay."0.1.1" = {
    futures = "0.1.25";
  };
  deps.remove_dir_all."0.5.1" = {
    winapi = "0.3.6";
  };
  deps.reqwest."0.8.8" = {
    bytes = "0.4.11";
    encoding_rs = "0.8.16";
    futures = "0.1.25";
    hyper = "0.11.27";
    hyper_tls = "0.1.4";
    libflate = "0.1.19";
    log = "0.4.6";
    mime_guess = "2.0.0";
    native_tls = "0.1.5";
    serde = "1.0.87";
    serde_json = "1.0.38";
    serde_urlencoded = "0.5.4";
    tokio_core = "0.1.17";
    tokio_io = "0.1.11";
    tokio_tls = "0.1.4";
    url = "1.7.2";
    uuid = "0.6.5";
  };
  deps.resolv_conf."0.6.2" = {
    hostname = "0.1.5";
    quick_error = "1.2.2";
  };
  deps.ring."0.13.5" = {
    untrusted = "0.6.2";
    cc = "1.0.29";
    lazy_static = "1.2.0";
    libc = "0.2.48";
  };
  deps.rustc_demangle."0.1.13" = {};
  deps.rustc_serialize."0.3.24" = {};
  deps.rustc_version."0.2.3" = {
    semver = "0.9.0";
  };
  deps.ryu."0.2.7" = {};
  deps.safemem."0.3.0" = {};
  deps.same_file."1.0.4" = {
    winapi_util = "0.1.2";
  };
  deps.schannel."0.1.14" = {
    lazy_static = "1.2.0";
    winapi = "0.3.6";
  };
  deps.scoped_tls."0.1.2" = {};
  deps.scopeguard."0.3.3" = {};
  deps.security_framework."0.1.16" = {
    core_foundation = "0.2.3";
    core_foundation_sys = "0.2.3";
    libc = "0.2.48";
    security_framework_sys = "0.1.16";
  };
  deps.security_framework_sys."0.1.16" = {
    core_foundation_sys = "0.2.3";
    libc = "0.2.48";
  };
  deps.semver."0.9.0" = {
    semver_parser = "0.7.0";
  };
  deps.semver_parser."0.7.0" = {};
  deps.serde."1.0.87" = {};
  deps.serde_derive."1.0.87" = {
    proc_macro2 = "0.4.27";
    quote = "0.6.11";
    syn = "0.15.26";
  };
  deps.serde_json."1.0.38" = {
    itoa = "0.4.3";
    ryu = "0.2.7";
    serde = "1.0.87";
  };
  deps.serde_urlencoded."0.5.4" = {
    dtoa = "0.4.3";
    itoa = "0.4.3";
    serde = "1.0.87";
    url = "1.7.2";
  };
  deps.serde_yaml."0.8.8" = {
    dtoa = "0.4.3";
    linked_hash_map = "0.5.1";
    serde = "1.0.87";
    yaml_rust = "0.4.2";
  };
  deps.sha1."0.6.0" = {};
  deps.signal_hook."0.1.7" = {
    arc_swap = "0.3.7";
    libc = "0.2.48";
  };
  deps.siphasher."0.2.3" = {};
  deps.slab."0.4.2" = {};
  deps.smallvec."0.6.8" = {
    unreachable = "1.0.0";
  };
  deps.socket2."0.3.8" = {
    cfg_if = "0.1.6";
    libc = "0.2.48";
    redox_syscall = "0.1.51";
    winapi = "0.3.6";
  };
  deps.stable_deref_trait."1.1.1" = {};
  deps.string."0.1.3" = {};
  deps.strsim."0.7.0" = {};
  deps.syn."0.15.26" = {
    proc_macro2 = "0.4.27";
    quote = "0.6.11";
    unicode_xid = "0.1.0";
  };
  deps.synstructure."0.10.1" = {
    proc_macro2 = "0.4.27";
    quote = "0.6.11";
    syn = "0.15.26";
    unicode_xid = "0.1.0";
  };
  deps.tempdir."0.3.7" = {
    rand = "0.4.6";
    remove_dir_all = "0.5.1";
  };
  deps.termion."1.5.1" = {
    libc = "0.2.48";
    redox_syscall = "0.1.51";
    redox_termios = "0.1.1";
  };
  deps.textwrap."0.10.0" = {
    unicode_width = "0.1.5";
  };
  deps.thread_local."0.3.6" = {
    lazy_static = "1.2.0";
  };
  deps.threadpool."1.7.1" = {
    num_cpus = "1.9.0";
  };
  deps.time."0.1.42" = {
    libc = "0.2.48";
    redox_syscall = "0.1.51";
    winapi = "0.3.6";
  };
  deps.tokio."0.1.15" = {
    bytes = "0.4.11";
    futures = "0.1.25";
    mio = "0.6.16";
    num_cpus = "1.9.0";
    tokio_codec = "0.1.1";
    tokio_current_thread = "0.1.4";
    tokio_executor = "0.1.6";
    tokio_fs = "0.1.5";
    tokio_io = "0.1.11";
    tokio_reactor = "0.1.8";
    tokio_sync = "0.1.1";
    tokio_tcp = "0.1.3";
    tokio_threadpool = "0.1.11";
    tokio_timer = "0.2.10";
    tokio_udp = "0.1.3";
    tokio_uds = "0.2.5";
  };
  deps.tokio_codec."0.1.1" = {
    bytes = "0.4.11";
    futures = "0.1.25";
    tokio_io = "0.1.11";
  };
  deps.tokio_core."0.1.17" = {
    bytes = "0.4.11";
    futures = "0.1.25";
    iovec = "0.1.2";
    log = "0.4.6";
    mio = "0.6.16";
    scoped_tls = "0.1.2";
    tokio = "0.1.15";
    tokio_executor = "0.1.6";
    tokio_io = "0.1.11";
    tokio_reactor = "0.1.8";
    tokio_timer = "0.2.10";
  };
  deps.tokio_current_thread."0.1.4" = {
    futures = "0.1.25";
    tokio_executor = "0.1.6";
  };
  deps.tokio_executor."0.1.6" = {
    crossbeam_utils = "0.6.5";
    futures = "0.1.25";
  };
  deps.tokio_fs."0.1.5" = {
    futures = "0.1.25";
    tokio_io = "0.1.11";
    tokio_threadpool = "0.1.11";
  };
  deps.tokio_io."0.1.11" = {
    bytes = "0.4.11";
    futures = "0.1.25";
    log = "0.4.6";
  };
  deps.tokio_reactor."0.1.8" = {
    crossbeam_utils = "0.6.5";
    futures = "0.1.25";
    lazy_static = "1.2.0";
    log = "0.4.6";
    mio = "0.6.16";
    num_cpus = "1.9.0";
    parking_lot = "0.7.1";
    slab = "0.4.2";
    tokio_executor = "0.1.6";
    tokio_io = "0.1.11";
  };
  deps.tokio_service."0.1.0" = {
    futures = "0.1.25";
  };
  deps.tokio_signal."0.2.7" = {
    futures = "0.1.25";
    mio = "0.6.16";
    tokio_executor = "0.1.6";
    tokio_io = "0.1.11";
    tokio_reactor = "0.1.8";
    libc = "0.2.48";
    mio_uds = "0.6.7";
    signal_hook = "0.1.7";
    winapi = "0.3.6";
  };
  deps.tokio_sync."0.1.1" = {
    futures = "0.1.25";
  };
  deps.tokio_tcp."0.1.3" = {
    bytes = "0.4.11";
    futures = "0.1.25";
    iovec = "0.1.2";
    mio = "0.6.16";
    tokio_io = "0.1.11";
    tokio_reactor = "0.1.8";
  };
  deps.tokio_threadpool."0.1.11" = {
    crossbeam = "0.6.0";
    crossbeam_channel = "0.3.8";
    crossbeam_deque = "0.6.3";
    crossbeam_utils = "0.6.5";
    futures = "0.1.25";
    log = "0.4.6";
    num_cpus = "1.9.0";
    rand = "0.6.5";
    slab = "0.4.2";
    tokio_executor = "0.1.6";
  };
  deps.tokio_timer."0.2.10" = {
    crossbeam_utils = "0.6.5";
    futures = "0.1.25";
    slab = "0.4.2";
    tokio_executor = "0.1.6";
  };
  deps.tokio_tls."0.1.4" = {
    futures = "0.1.25";
    native_tls = "0.1.5";
    tokio_core = "0.1.17";
    tokio_io = "0.1.11";
  };
  deps.tokio_udp."0.1.3" = {
    bytes = "0.4.11";
    futures = "0.1.25";
    log = "0.4.6";
    mio = "0.6.16";
    tokio_codec = "0.1.1";
    tokio_io = "0.1.11";
    tokio_reactor = "0.1.8";
  };
  deps.tokio_uds."0.2.5" = {
    bytes = "0.4.11";
    futures = "0.1.25";
    iovec = "0.1.2";
    libc = "0.2.48";
    log = "0.4.6";
    mio = "0.6.16";
    mio_uds = "0.6.7";
    tokio_codec = "0.1.1";
    tokio_io = "0.1.11";
    tokio_reactor = "0.1.8";
  };
  deps.tower_service."0.1.0" = {
    futures = "0.1.25";
  };
  deps.trust_dns_proto."0.5.0" = {
    byteorder = "1.3.1";
    failure = "0.1.5";
    futures = "0.1.25";
    idna = "0.1.5";
    lazy_static = "1.2.0";
    log = "0.4.6";
    rand = "0.5.6";
    smallvec = "0.6.8";
    socket2 = "0.3.8";
    tokio_executor = "0.1.6";
    tokio_io = "0.1.11";
    tokio_reactor = "0.1.8";
    tokio_tcp = "0.1.3";
    tokio_timer = "0.2.10";
    tokio_udp = "0.1.3";
    url = "1.7.2";
  };
  deps.trust_dns_proto."0.6.3" = {
    byteorder = "1.3.1";
    failure = "0.1.5";
    futures = "0.1.25";
    idna = "0.1.5";
    lazy_static = "1.2.0";
    log = "0.4.6";
    rand = "0.5.6";
    smallvec = "0.6.8";
    socket2 = "0.3.8";
    tokio_executor = "0.1.6";
    tokio_io = "0.1.11";
    tokio_reactor = "0.1.8";
    tokio_tcp = "0.1.3";
    tokio_timer = "0.2.10";
    tokio_udp = "0.1.3";
    url = "1.7.2";
  };
  deps.trust_dns_resolver."0.10.3" = {
    cfg_if = "0.1.6";
    failure = "0.1.5";
    futures = "0.1.25";
    lazy_static = "1.2.0";
    log = "0.4.6";
    lru_cache = "0.1.1";
    resolv_conf = "0.6.2";
    smallvec = "0.6.8";
    tokio = "0.1.15";
    trust_dns_proto = "0.6.3";
    ipconfig = "0.1.9";
  };
  deps.try_lock."0.1.0" = {};
  deps.ucd_util."0.1.3" = {};
  deps.unicase."1.4.2" = {
    version_check = "0.1.5";
  };
  deps.unicase."2.2.0" = {
    version_check = "0.1.5";
  };
  deps.unicode_bidi."0.3.4" = {
    matches = "0.1.8";
  };
  deps.unicode_normalization."0.1.8" = {
    smallvec = "0.6.8";
  };
  deps.unicode_width."0.1.5" = {};
  deps.unicode_xid."0.1.0" = {};
  deps.unreachable."1.0.0" = {
    void = "1.0.2";
  };
  deps.untrusted."0.6.2" = {};
  deps.url."1.7.2" = {
    encoding = "0.2.33";
    idna = "0.1.5";
    matches = "0.1.8";
    percent_encoding = "1.0.1";
  };
  deps.utf8_ranges."1.0.2" = {};
  deps.uuid."0.1.18" = {
    rand = "0.3.23";
    rustc_serialize = "0.3.24";
  };
  deps.uuid."0.6.5" = {
    cfg_if = "0.1.6";
    rand = "0.4.6";
  };
  deps.uuid."0.7.2" = {
    rand = "0.6.5";
  };
  deps.v_escape."0.3.2" = {
    v_escape_derive = "0.2.1";
    version_check = "0.1.5";
  };
  deps.v_escape_derive."0.2.1" = {
    nom = "4.2.0";
    proc_macro2 = "0.4.27";
    quote = "0.6.11";
    syn = "0.15.26";
  };
  deps.v_htmlescape."0.3.2" = {
    cfg_if = "0.1.6";
    v_escape = "0.3.2";
    version_check = "0.1.5";
  };
  deps.vcpkg."0.2.6" = {};
  deps.vec_map."0.8.1" = {};
  deps.version_check."0.1.5" = {};
  deps.void."1.0.2" = {};
  deps.walkdir."2.2.7" = {
    same_file = "1.0.4";
    winapi = "0.3.6";
    winapi_util = "0.1.2";
  };
  deps.want."0.0.4" = {
    futures = "0.1.25";
    log = "0.4.6";
    try_lock = "0.1.0";
  };
  deps.widestring."0.2.2" = {};
  deps.winapi."0.2.8" = {};
  deps.winapi."0.3.6" = {
    winapi_i686_pc_windows_gnu = "0.4.0";
    winapi_x86_64_pc_windows_gnu = "0.4.0";
  };
  deps.winapi_build."0.1.1" = {};
  deps.winapi_i686_pc_windows_gnu."0.4.0" = {};
  deps.winapi_util."0.1.2" = {
    winapi = "0.3.6";
  };
  deps.winapi_x86_64_pc_windows_gnu."0.4.0" = {};
  deps.winreg."0.5.1" = {
    winapi = "0.3.6";
  };
  deps.winutil."0.1.1" = {
    winapi = "0.3.6";
  };
  deps.ws2_32_sys."0.2.1" = {
    winapi = "0.2.8";
    winapi_build = "0.1.1";
  };
  deps.yaml_rust."0.4.2" = {
    linked_hash_map = "0.5.1";
  };
}
