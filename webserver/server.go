package main

import (
	"fmt"
	"net"
	"os"
	"strings"
)

var id = 0
var routes = make(map[string]func(*Request) Response)

type Request struct {
	Method  string
	Path    string
	Params  map[string]string
	Headers map[string]string
	Body    string
}

type Response struct {
	Status  int
	Body    []byte
	Headers map[string]string
}

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
		go process_connection(conn)
	}
}

func process_connection(conn net.Conn) {
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

	request, err := parse_request(buffer)
	if err != nil {
		println("ERROR: Invalid http request")
		return
	}

	fmt.Printf("INFO: Recieved request: %s %s\n", request.Method, request.Path)

	response := route(request)

	http_response := fmt.Sprintf("HTTP/1.1 %d OK\r\n\r\n%s\r\n", response.Status, response.Body)
	n, err = conn.Write([]byte(http_response))
	if err != nil {
		println("ERROR: conn.Write: ", err)
		return
	}
}

func parse_request(buffer []byte) (*Request, error) {
	message := string(buffer[:])
	start_string, rest, found := strings.Cut(message, "\r\n")

	if !found {
		return nil, fmt.Errorf("Invalid http request")
	}

	start_string_parts := strings.Split(start_string, " ")
	method := start_string_parts[0]
	path_with_query_string := start_string_parts[1]

	path, query_string, found_query_params := strings.Cut(path_with_query_string, "?")

	var params = make(map[string]string)

	if found_query_params {
		for _, query_param := range strings.Split(query_string, "&") {
			key, value, found := strings.Cut(query_param, "=")
			if found {
				params[key] = value
			}
		}
	}

	headers_string, body, _ := strings.Cut(rest, "\r\n\r\n")

	var headers = make(map[string]string)
	for _, header_string := range strings.Split(headers_string, "\r\n") {
		key, value, found := strings.Cut(header_string, ": ")
		if found {
			headers[key] = value
		}
	}

	request := Request{method, path, params, headers, body}
	return &request, nil
}

func mount(path string, handler func(*Request) Response) {
	_, exist := routes[path]
	if exist {
		println("WARN: handler for route %s is already defined", path)
	}

	routes[path] = handler
}

func route(request *Request) Response {
	handler, handler_found := routes[request.Path]
	if handler_found {
		return handler(request)
	}

	println("handler not found")
	fmt.Printf("ROUTES: %v", routes)

	return Response{Status: 404, Body: []byte("Not found")}
}

func UNUSED(x ...interface{}) {}
