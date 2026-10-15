# Assembly layer (D66 - five-part order modeled on the VisiaEngine consumer shape).
# Version single-source wiring lands with M2 (workspace Cargo.toml does not exist yet);
# binding lanes + examples + install ride this graph later per docs/stage-1-plan.md.

cmake_minimum_required(VERSION 3.25)   # PolyOrch embed floor (D66; VisiaEngine D39 equivalent)
project(modukit LANGUAGES CXX)       # CXX identity required before the PolyOrch embed (PlatformSupport contract, observed first-red M1; VisiaEngine root same shape)

enable_testing()                       # must run before ALL add_subdirectory calls (measured lesson)

include(cmake/ModuKitPolyOrch.cmake)
modukit_ensure_polyorch()              # sentinel embed: reuse superbuild copy first-wins, else gitlink pin

# add_subdirectory(crates/...) lanes + install/export arrive with M2/M3 - see docs/stage-1-plan.md
