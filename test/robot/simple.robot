*** Settings ***
Documentation     A test suite for simple crabfiles that take no args, always return 0, and print some output.
Resource          keywords/all.robot
Library           String

*** Keywords ***
The Crabfile "${crabfile}" is built and outputs "${result}"
        The Crabfile "${crabfile}" is built
        ${crabfile_name} =  Fetch From Left  ${crabfile}  .
        The "${crabfile_name}" Crab application is run successfully
        The last process printed "${result}"

*** Test Cases ***
Run Simple Crabfiles
    [Template]  The Crabfile "${crabfile}" is built and outputs "${result}"
    hello_world.crab                Hello, world!
    addition.crab                   42
    struct.crab                     Phillip went to the gym
    optional_param.crab             foobar
    func_tmpl.crab                  contained!, 1337
