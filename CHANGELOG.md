# Changelog

## [0.8.1](https://github.com/majksa-dev/rust-gateway/compare/v0.8.0...v0.8.1) (2024-07-29)


### Bug Fixes

* add any router ([bc27e8a](https://github.com/majksa-dev/rust-gateway/commit/bc27e8a5c2bc437543d8bc9f5c52c139e0bd4189))
* allow dynamically and statically providing hosts ([0af3d76](https://github.com/majksa-dev/rust-gateway/commit/0af3d767b84a7e634b8fd5a11fc57ea1464b73f3))
* **cache:** add endpoint custom header back ([4a80f6e](https://github.com/majksa-dev/rust-gateway/commit/4a80f6e9dec8981a2324079963e93bdabd1931d1))
* **deps:** bump serde_json from 1.0.120 to 1.0.121 ([052011d](https://github.com/majksa-dev/rust-gateway/commit/052011da0a6e0cd9ab1d72976ecfd6659d6aeddd))

## [0.8.0](https://github.com/majksa-dev/rust-gateway/compare/v0.7.0...v0.8.0) (2024-07-29)


### Features

* override host header in tcp process ([2697d07](https://github.com/majksa-dev/rust-gateway/commit/2697d07652b7af3d507158ba89a2d1fe3a536b01))

## [0.7.0](https://github.com/majksa-dev/rust-gateway/compare/v0.6.4...v0.7.0) (2024-07-28)


### Features

* **auth:** add endpoints based authentication and authorization - e.g. for OIDC based providers (google, etc.) ([a1b73e0](https://github.com/majksa-dev/rust-gateway/commit/a1b73e0a533f5ec37b1a3cddac51bd9232892a1b))

## [0.6.4](https://github.com/majksa-dev/rust-gateway/compare/v0.6.3...v0.6.4) (2024-07-27)


### Bug Fixes

* **deps:** bump pingora-cache from 0.2.0 to 0.3.0 ([8c46067](https://github.com/majksa-dev/rust-gateway/commit/8c46067d4101601f9373f766f7793c03501312d0))
* **deps:** bump tokio from 1.38.0 to 1.39.2 ([15b31ab](https://github.com/majksa-dev/rust-gateway/commit/15b31abc44580ba9ceaa0c015ba8e7764c555ec9))
* **deps:** bump wiremock from 0.6.0 to 0.6.1 ([e94befa](https://github.com/majksa-dev/rust-gateway/commit/e94befa30751b38c5d4fadc746a01d1c84261013))
* tls server special cases ([ad833e6](https://github.com/majksa-dev/rust-gateway/commit/ad833e636fed063c0a8ee07575c7145ed93776f9))

## [0.6.3](https://github.com/majksa-dev/rust-gateway/compare/v0.6.2...v0.6.3) (2024-07-10)


### Bug Fixes

* convert take input arg to u64 ([b8129da](https://github.com/majksa-dev/rust-gateway/commit/b8129da48dad9da68f12d4977a7c3a7bc2b277cb))
* **deps:** bump async-trait from 0.1.80 to 0.1.81 ([fe1b544](https://github.com/majksa-dev/rust-gateway/commit/fe1b5447e38bf0e3e7c2a9a45a5f9ca9ef539af6))
* **deps:** bump io from 0.2.2 to 0.3.0 ([d24d9f0](https://github.com/majksa-dev/rust-gateway/commit/d24d9f0ded9dfd32628ea2cf4aa13713a04394bd))
* **deps:** bump serde from 1.0.203 to 1.0.204 ([54d774e](https://github.com/majksa-dev/rust-gateway/commit/54d774ecafee49048f22ccaa1d486bec45ff7213))
* flush after writing ([dd22eb1](https://github.com/majksa-dev/rust-gateway/commit/dd22eb18f9704242a232216bce9b120b43dfeda8))
* improve zero copy tcp ([0dada80](https://github.com/majksa-dev/rust-gateway/commit/0dada8098371814a0cdb631214470304dc6c6d1b))
* limit input body ([430666c](https://github.com/majksa-dev/rust-gateway/commit/430666c5436eac50b76781582214050e53775455))
* remove buffering when reading response ([8df4e45](https://github.com/majksa-dev/rust-gateway/commit/8df4e4529c261d4f5be3ef3c3b9309e6f36f3ec2))
* use more reliable status line reader ([6f1dfcd](https://github.com/majksa-dev/rust-gateway/commit/6f1dfcd777ddfa2804b652c439f0b2dee93f000c))
* wait for response from origin server ([404c1a8](https://github.com/majksa-dev/rust-gateway/commit/404c1a858f73bc6c01778bae5a996902558f3b4c))

## [0.6.2](https://github.com/majksa-dev/rust-gateway/compare/v0.6.1...v0.6.2) (2024-07-04)


### Bug Fixes

* allow specifying host as hostname ([4b50b63](https://github.com/majksa-dev/rust-gateway/commit/4b50b6321f5ea2e382dc4c144f7d5acaa8eae021))

## [0.6.1](https://github.com/majksa-dev/rust-gateway/compare/v0.6.0...v0.6.1) (2024-07-04)


### Bug Fixes

* cors origin auth and better error logging ([9742964](https://github.com/majksa-dev/rust-gateway/commit/97429640bb73b5bf914b49574dedf2476f43dbee))

## [0.6.0](https://github.com/majksa-dev/rust-gateway/compare/v0.5.6...v0.6.0) (2024-07-04)


### Features

* enable tls for entrypoint communication (client -&gt; gateway). origin communication (gateway -> server) still only supports HTTP ([f13fb28](https://github.com/majksa-dev/rust-gateway/commit/f13fb28cc375daeb32a773de147464474075be7a))

## [0.5.6](https://github.com/majksa-dev/rust-gateway/compare/v0.5.5...v0.5.6) (2024-07-04)


### Bug Fixes

* **performance:** pass middlewares as iterator ([f2d1046](https://github.com/majksa-dev/rust-gateway/commit/f2d10461059aed9972a138be476fbc4ca44281d7))

## [0.5.5](https://github.com/majksa-dev/rust-gateway/compare/v0.5.4...v0.5.5) (2024-07-03)


### Bug Fixes

* **deps:** bump serde_json from 1.0.118 to 1.0.120 ([85cb9f1](https://github.com/majksa-dev/rust-gateway/commit/85cb9f182676fdef524c0d30852d405159ba3cc3))

## [0.5.4](https://github.com/majksa-dev/rust-gateway/compare/v0.5.3...v0.5.4) (2024-06-28)


### Bug Fixes

* skip to next middleware ([e75bc6e](https://github.com/majksa-dev/rust-gateway/commit/e75bc6ec1b320308ef9d8e49ec8bc601f22cd2d6))

## [0.5.3](https://github.com/majksa-dev/rust-gateway/compare/v0.5.2...v0.5.3) (2024-06-27)


### Bug Fixes

* implement from iterators for middlewares, origins and routers ([c523c2f](https://github.com/majksa-dev/rust-gateway/commit/c523c2f9f90d137b55387480253c5080126d22ec))

## [0.5.2](https://github.com/majksa-dev/rust-gateway/compare/v0.5.1...v0.5.2) (2024-06-27)


### Bug Fixes

* rename all feature to full ([2615765](https://github.com/majksa-dev/rust-gateway/commit/26157658065d92636923dd1f31ccfe67c10a535e))

## [0.5.1](https://github.com/majksa-dev/rust-gateway/compare/v0.5.0...v0.5.1) (2024-06-27)


### Bug Fixes

* re-export router builder ([0f0a865](https://github.com/majksa-dev/rust-gateway/commit/0f0a8656d90549b733246f55bf38a804672b6156))

## [0.5.0](https://github.com/majksa-dev/rust-gateway/compare/v0.4.1...v0.5.0) (2024-06-26)


### Features

* add convertors from raw config ([59790e6](https://github.com/majksa-dev/rust-gateway/commit/59790e60e54460455f6d23d5ad87ec18a3be5db6))
* add jwt token based authentication ([adda6ee](https://github.com/majksa-dev/rust-gateway/commit/adda6ee4f23f38739b9052b0e76777d1baa57472))
* **auth:** add basic auth middleware ([e5a256f](https://github.com/majksa-dev/rust-gateway/commit/e5a256ff3698de84deaa82df1dcea45c12366153))
* **auth:** add jwt token validation for oidc ([f7e1949](https://github.com/majksa-dev/rust-gateway/commit/f7e1949a022a6d6ed874808bff1c2584c38bb1b7))
* **cors:** allow switching between allow all, even without origin header ([cb08ad3](https://github.com/majksa-dev/rust-gateway/commit/cb08ad31a5de8a92e66ff3b3b7b7e22127141d5e))
* split configuration structs from app context, to decrease memory allocation and time ([4dfac4e](https://github.com/majksa-dev/rust-gateway/commit/4dfac4e539b15b41c5fc376821de7fd4b8741683))
* split middlewares into features, separated tests, etc. ([7c5748e](https://github.com/majksa-dev/rust-gateway/commit/7c5748ec0d2e56523e24f9ba4629f7041b27ba79))


### Bug Fixes

* **cors:** return forbidden instead of bad request ([20ab949](https://github.com/majksa-dev/rust-gateway/commit/20ab949b413b855ea1237de42af46d3c0ea5a89b))
* **cors:** when no origins specified, allow all ([7d90f6b](https://github.com/majksa-dev/rust-gateway/commit/7d90f6baa8558cf0664f2c04ef08e3a5cd230ab1))
* **deps:** bump io from 0.2.1 to 0.2.2 ([389644b](https://github.com/majksa-dev/rust-gateway/commit/389644be59a19230defbe41e8adb122cfa3e1cde))
* do not throw error when inserting headers ([8b5b518](https://github.com/majksa-dev/rust-gateway/commit/8b5b518328b22873379b31d2b9bb15b2c123f129))
* **http:** extract common http headers ([db8ad9a](https://github.com/majksa-dev/rust-gateway/commit/db8ad9a42f59e35c95e3e6a683714851204dd440))
* lib.rs example usage ([545411e](https://github.com/majksa-dev/rust-gateway/commit/545411e3b516417fcd1f829c412c149e229ee5ea))

## [0.4.1](https://github.com/majksa-dev/rust-gateway/compare/v0.4.0...v0.4.1) (2024-06-22)


### Bug Fixes

* allow user to specify rate limits based on token ([1290cc9](https://github.com/majksa-dev/rust-gateway/commit/1290cc9594a31356f02df5da28d9708d2e7e6ca5))

## [0.4.0](https://github.com/majksa-dev/rust-gateway/compare/v0.3.0...v0.4.0) (2024-06-22)


### Features

* add caching middleware ([#23](https://github.com/majksa-dev/rust-gateway/issues/23)) ([682671a](https://github.com/majksa-dev/rust-gateway/commit/682671a62fb37edcd7c3ca711cd3780e3c83c8b1))

## [0.3.0](https://github.com/majksa-dev/rust-gateway/compare/v0.2.2...v0.3.0) (2024-06-22)


### Features

* copy tcp stream using zero copy on linux ([#22](https://github.com/majksa-dev/rust-gateway/issues/22)) ([9ac8d30](https://github.com/majksa-dev/rust-gateway/commit/9ac8d308db90d808a6b8bd7a6e4054957f6f3a63))
* implement custom rate limiter ([ce2d725](https://github.com/majksa-dev/rust-gateway/commit/ce2d7259244583c3f871f2fe1a02f116812d2ff9))
* rethink the endpoint router, reimplement cors policy, implement rate limiting ([f4a7683](https://github.com/majksa-dev/rust-gateway/commit/f4a7683760097cf46a928e08e2561e20bdacdd4d))


### Bug Fixes

* do not setup panic hook in tests ([be2cc0a](https://github.com/majksa-dev/rust-gateway/commit/be2cc0ae4dbf983ff783c737676d0dd8b21c6886))
* make code stable by removing unwraps ([cec3429](https://github.com/majksa-dev/rust-gateway/commit/cec3429e82253ec00b26d42f76830b01a495cdca))
* middlewares direction from lowest number to largest ([2239504](https://github.com/majksa-dev/rust-gateway/commit/223950419bb87362a8766521d81d577e09a3a07a))
* pass owned halfs of tcp stream instead of Arc ([be50002](https://github.com/majksa-dev/rust-gateway/commit/be50002bfc3f0117a86081cdcf1645d77f3e8f68))
* **perf:** implement io::copy using std and sendfile ([8c59b84](https://github.com/majksa-dev/rust-gateway/commit/8c59b84fe3d197630d048920d3209fab94225c7d))
* provider constructor for Cors middleware ([73af671](https://github.com/majksa-dev/rust-gateway/commit/73af67191f7b5c2ce6915a70630f9f3d7322ca80))
* remove beta support ([d39ab31](https://github.com/majksa-dev/rust-gateway/commit/d39ab31b4f52cb3f28a5133bf7fe82153682736e))
* return result from rate limit datastore ([80a4c7b](https://github.com/majksa-dev/rust-gateway/commit/80a4c7bb5eb30cd868cbd683394240205bf3dfbb))

## [0.2.2](https://github.com/majksa-dev/rust-gateway/compare/v0.2.1...v0.2.2) (2024-06-13)


### Bug Fixes

* accept moved values ([c0d407a](https://github.com/majksa-dev/rust-gateway/commit/c0d407a3b207d901b5d4f0f8d9d83da6c9184d84))

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
