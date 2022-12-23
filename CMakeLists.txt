project(vrisc)

set(SRC vrisc/main.c)
set(SRC ${SRC} vrisc/memc.c)
set(SRC ${SRC} vrisc/core/vrisc.c)
set(SRC ${SRC} vrisc/core/base.c)

add_executable(vrisc ${SRC})
add_executable(vas vas/vas.cc)
