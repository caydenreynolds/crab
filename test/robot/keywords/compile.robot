*** Settings ***
Documentation     Resources for running processes.
...               Results may be checked immediately and/or later
...               All stdout and stderr is forwarded to the log file
Resource          processes.robot
Library           Process
Library           String


*** Keywords ***

The Crabfile "${crabfile}" is built
    ${crabfile_name} =  Fetch From Left  ${crabfile}  .
    IF  "${VERBOSE}" == "TRUE"
        The following command is run:  ${CRABC}  -c  ${CBUILTINS_DIR}  -o  ${TARGET_DIR}/${crabfile_name}.exe  --verify  -v  ${CRAB_SRC}/${crabfile}  ${CRAB_STD}
    ELSE
        The following command is run:  ${CRABC}  -c  ${CBUILTINS_DIR}  -o  ${TARGET_DIR}/${crabfile_name}.exe  --verify  ${CRAB_SRC}/${crabfile}  ${CRAB_STD}
    END

The "${exe}" Crab application is run successfully
    The "${exe}" Crab application is run with exit code 0

The "${exe}" Crab application is run with exit code ${ec}
    The following command is run:  ${TARGET_DIR}/${exe}.exe  return_code=${ec}

The "${exe}" Crab application is started
    ${process} =  Start Process  ${TARGET_DIR}/${exe}.exe
    Return from Keyword  ${process}
