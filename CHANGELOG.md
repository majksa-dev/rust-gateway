# Changelog

## [0.2.1](https://github.com/majksa-dev/rust-gateway/compare/v0.2.0...v0.2.1) (2024-06-13)


### Bug Fixes

* improve cors config ([a2de8e5](https://github.com/majksa-dev/rust-gateway/commit/a2de8e5f8f5940d8e83efb8387f16aed3c80795e))

## [0.2.0](https://github.com/majksa-dev/rust-gateway/compare/v0.1.3...v0.2.0) (2024-06-13)


### Features

* implement custom server, add cors logic, remove old libraries and write tests using testing utils ([859bfdc](https://github.com/majksa-dev/rust-gateway/commit/859bfdc9cb9a1adccedf50bca64c13369784b54f))


### Bug Fixes

* **deps:** bump proc-macro2 from 1.0.82 to 1.0.85 ([2d6e6c3](https://github.com/majksa-dev/rust-gateway/commit/2d6e6c333fb492e6fe35b084398adfe5b3fe23b4))
* **deps:** bump tokio from 1.37.0 to 1.38.0 ([10ffd20](https://github.com/majksa-dev/rust-gateway/commit/10ffd20d4d8db2df6329ca05069e2ee0c99dcdb6))
* remove unused dependencies ([a006029](https://github.com/majksa-dev/rust-gateway/commit/a0060294f1cab8d48149180b6a75d9d4eb70652f))

## [0.1.3](https://github.com/majksa-dev/rust-gateway/compare/v0.1.2...v0.1.3) (2024-06-11)


### Bug Fixes

* handle returning status code error as ok ([30ae6a6](https://github.com/majksa-dev/rust-gateway/commit/30ae6a69a38d04ae6ff7f8818549d7d161a06b74))

## [0.1.2](https://github.com/majksa-dev/rust-gateway/compare/v0.1.1...v0.1.2) (2024-06-11)


### Bug Fixes

* server imports ([78118b2](https://github.com/majksa-dev/rust-gateway/commit/78118b2a4b5fe2d116d72612a478441262701968))

## [0.1.1](https://github.com/majksa-dev/rust-gateway/compare/v0.1.0...v0.1.1) (2024-06-11)


### Bug Fixes

* update imports ([b0ae9b4](https://github.com/majksa-dev/rust-gateway/commit/b0ae9b465e4577a913e447b435c5c1b6fbe43cbf))

## 0.1.0 (2024-06-11)


### Features

* add endpoint id to context ([22ce8c5](https://github.com/majksa-dev/rust-gateway/commit/22ce8c540b6e9d45b834b16176ae717cd04f47f3))
* add helper utils ([bf7854e](https://github.com/majksa-dev/rust-gateway/commit/bf7854e40bdeb9ab3992f207bd02b28bde7acd1c))
* add middleware context ([77205b4](https://github.com/majksa-dev/rust-gateway/commit/77205b4758184cdf20b8a2f2cad138c3651c9dd4))
* add mutable context for middlewares ([1e8e531](https://github.com/majksa-dev/rust-gateway/commit/1e8e531b913907872be673005460b490c0d2485c))
* cors middleware ([492a799](https://github.com/majksa-dev/rust-gateway/commit/492a79901d93be9d8520d0bb1ebaf5bf8e24b54c))
* customizable origin server ([aa87240](https://github.com/majksa-dev/rust-gateway/commit/aa872404277ec5b1ce7bd138e4a363555e152313))
* improve middleware filter function to return Response ([75aced7](https://github.com/majksa-dev/rust-gateway/commit/75aced722490a2cbc0f059edcd27a4a475a7a8f3))
* **middleware:** add filter, modify_response and modify_request methods ([4fb1b50](https://github.com/majksa-dev/rust-gateway/commit/4fb1b503c0f6234ed4a052cc2364b75147147c8f))
* replace pingora with custom middleware server ([6ffcf94](https://github.com/majksa-dev/rust-gateway/commit/6ffcf944821d5da329c1acba72e0cc22dfab3d68))
* server and gateway entrypoint functionality ([278c011](https://github.com/majksa-dev/rust-gateway/commit/278c0115c8c6588093a253f254d5e6a9a84fb589))
* setup entrypoint upstream peer connector ([513c52f](https://github.com/majksa-dev/rust-gateway/commit/513c52fd78d0bd490214ef3ec4fae07ac084083b))
* simple server builder ([97e3b55](https://github.com/majksa-dev/rust-gateway/commit/97e3b55a466c30be9357a9e7754716b341c7905a))


### Bug Fixes

* accept unboxed values in builder ([e41d1cf](https://github.com/majksa-dev/rust-gateway/commit/e41d1cf31c8f9f5edaf39d613f10d253ed574d0e))
* **deps:** bump serde from 1.0.201 to 1.0.203 ([#5](https://github.com/majksa-dev/rust-gateway/issues/5)) ([847a26d](https://github.com/majksa-dev/rust-gateway/commit/847a26deeba57c9d5ccaea5323a3801c918480b0))
* give user the option to optionally provide host ([b2b8d86](https://github.com/majksa-dev/rust-gateway/commit/b2b8d866bdefc829d6816aecc45b56ea5c28994e))
* **middleware:** use temporarily work with generic context ([31fe8e4](https://github.com/majksa-dev/rust-gateway/commit/31fe8e498edd2c9500cfc3c4accd3d8a5f338611))
* remove unused utils ([0a617fd](https://github.com/majksa-dev/rust-gateway/commit/0a617fdbd16761f6242f1573ae11cc63883f2129))
