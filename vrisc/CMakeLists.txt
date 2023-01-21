aux_source_directory(. MAIN)
aux_source_directory(core CORE)
aux_source_directory(debug DBG)
aux_source_directory(tools TLS)

set(SRC ${MAIN} ${CORE} ${DBG} ${TLS})

add_executable(vcore ${SRC})
