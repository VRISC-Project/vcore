aux_source_directory(. MAIN)
aux_source_directory(core CORE)
aux_source_directory(debug DBG)

set(SRC ${MAIN} ${CORE} ${DBG})

add_executable(vrisc ${SRC})
