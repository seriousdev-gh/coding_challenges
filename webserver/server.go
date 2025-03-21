package main

import (
	"fmt"
	"net"
	"os"
	"strings"
)

var id = 0

func server_start(host string, port string) {
	socket, err := net.Listen("tcp", fmt.Sprintf("%s:%s", host, port))
	if err != nil {
		println("Error creating socket: ", err)
		os.Exit(1)
	}
	defer socket.Close()

	fmt.Printf("INFO: Server ready to accept connection on %s:%s\n", host, port)

	for {
		conn, err := socket.Accept()
		if err != nil {
			println("ERROR: socket.Accept:", err)
			break
		}
		id += 1
		go process_connection(conn, id)
	}
}

func process_connection(conn net.Conn, id int) {
	defer conn.Close()
	var n int
	UNUSED(n)
	var err error

	var buffer = make([]byte, 1024) // TODO: support request larger than 1024 bytes

	n, err = conn.Read(buffer)
	if err != nil {
		println("ERROR: conn.Read: ", err)
		return
	}

	message := string(buffer[:])
	start_string, rest, found := strings.Cut(message, "\r\n")
	UNUSED(rest)
	if !found {
		println("ERROR: Invalid http request")
		return
	}
	start_string_parts := strings.Split(start_string, " ")
	for i, part := range start_string_parts {
		start_string_parts[i] = strings.TrimSpace(part)
	}
	method := start_string_parts[0]
	path := start_string_parts[1]

	fmt.Printf("[%d] INFO: Recieved request: %s %s\n", id, method, path)

	status, body := serve(method, path)

	response := fmt.Sprintf("HTTP/1.1 %d OK\r\n\r\n%s\r\n", status, body)
	n, err = conn.Write([]byte(response))
	if err != nil {
		println("ERROR: conn.Write: ", err)
		return
	}
}

func serve(method string, path string) (int, []byte) {
	if path == "/" {
		path = "/index.html"
	}
	content, err := os.ReadFile(fmt.Sprintf("www%s", path))
	if err != nil {
		fmt.Printf("WARN: os.ReadFile: %v\n", err)
		return 404, []byte("Not found")
	}

	return 200, content
}

func UNUSED(x ...interface{}) {}
