import io
import time
import subprocess
import sys


import sys
import subprocess
from threading import Thread 
import time
from inspect import signature

def get_commands():
    commands = [
        bench,
        build_quiet,
    ]
    return dict(map(lambda command: (command.__name__, command), commands))



def main():
    commands = get_commands()
    if len(sys.argv) > 1 and sys.argv[1] in commands:
        command = commands[sys.argv[1]]
        parameters = sys.argv[2:]
        if len(parameters) != len(signature(command).parameters):
            usage()
        commands[sys.argv[1]](*sys.argv[2:])
    else:
        usage()



def usage():
    print("blah blah")
    exit()


def build_quiet(profile):
    process = subprocess.Popen(["cargo", "rustc", "--{}".format(profile)], stderr = subprocess.PIPE)
    print("building {}".format(profile))
    if process.wait() != 0:
        print("cargo build failed:")
        print(process.stderr.read())
    else:
        print("all good (:")

def bench():
    stderr_buffer = []
    stdout_buffer = []
    process = subprocess.Popen(["cargo", "bench", "--", "--noplot"], stderr = subprocess.PIPE)



    def reader(io, buffer):
        while True:
            sys.stdout.buffer.write(io.readline())

    thread = Thread(target = reader, args = (process.stderr, stderr_buffer))
    thread.daemon=True
    thread.start()

    process.wait()



if __name__ == "__main__":
    main()