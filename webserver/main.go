package main

import (
	"fmt"
	"net"
	"strings"
)

func main() {
	ln, err := net.Listen("tcp", "127.0.0.1:3000")
	if err != nil {
		println("Error creating socket: ", err)
	}

	println("Ready to accept connections")
	for {
		var n int
		var err error
		conn, err := ln.Accept()
		if err != nil {
			println("Error while accepting connection: ", err)
			conn.Close()
			continue
		}

		var buffer = make([]byte, 1024)

		n, err = conn.Read(buffer)
		if err != nil {
			println("Error while reading from socket: ", err)
			conn.Close()
			continue
		}

		println("Got ", n, " bytes from socket")
		message := string(buffer[:])
		first_line, rest, found := strings.Cut(message, "\r\n")
		if !found {
			println("Invalid http request")
			conn.Close()
			continue
		}
		info := strings.Split(first_line, " ")
		info = Map(info, strings.TrimSpace)

		fmt.Printf("Method: '%s', Path: '%s', '%s'", info[0], info[1], info[2])

		println("Body: ", rest)

		response := fmt.Sprintf("HTTP/1.1 200 OK\r\n\r\nRequested path: %s\r\n", info[1])

		n, err = conn.Write([]byte(response))
		if err != nil {
			println("Error while reading from socket: ", err)
			conn.Close()
			continue
		}
		println("Written ", n, " bytes to socket")
		conn.Close()
	}
}

func Map(vs []string, f func(string) string) []string {
	vsm := make([]string, len(vs))
	for i, v := range vs {
		vsm[i] = f(v)
	}
	return vsm
}
