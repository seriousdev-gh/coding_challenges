package main

import (
	"fmt"
	"html"
	"net"
	"net/url"
	"os"
	"strings"
)

var id = 0
var routes []Route

type Route struct {
	Method  string
	Path    string
	Handler func(*Request) Response
}

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
	var err error

	var buffer = make([]byte, 2048) // TODO: support request larger than 1024 bytes

	n, err = conn.Read(buffer)
	if err != nil {
		println("ERROR: conn.Read: ", err)
		return
	}

	request, err := parse_request(buffer, n)
	if err != nil {
		println("ERROR: Invalid http request")
		return
	}

	fmt.Printf("INFO: Recieved request: %s %s\n", request.Method, request.Path)

	response := route(request)

	var headers strings.Builder

	for key, value := range response.Headers {
		headers.WriteString(key)
		headers.WriteString(": ")
		headers.WriteString(value)
		headers.WriteString("\r\n")
	}

	http_response := fmt.Sprintf("HTTP/1.1 %d OK\r\n%s\r\n%s\r\n", response.Status, headers.String(), response.Body)
	n, err = conn.Write([]byte(http_response))
	if err != nil {
		println("ERROR: conn.Write: ", err)
		return
	}
}

func parse_request(buffer []byte, size int) (*Request, error) {
	message := string(buffer[:size])
	start_string, rest, found := strings.Cut(message, "\r\n")

	if !found {
		return nil, fmt.Errorf("Invalid http request")
	}

	var params = make(map[string]string)

	start_string_parts := strings.Split(start_string, " ")
	method := start_string_parts[0]
	path_with_query_string := start_string_parts[1]
	path, query_string, found_query_params := strings.Cut(path_with_query_string, "?")

	if found_query_params {
		append_url_encoded_params(params, query_string)
	}

	header_part, body_part, _ := strings.Cut(rest, "\r\n\r\n")
	headers := parse_headers(header_part)

	if headers["Content-Type"] == "application/x-www-form-urlencoded" {
		encoded_params, _, _ := strings.Cut(body_part, "\r\n")
		append_url_encoded_params(params, encoded_params)
	}

	request := Request{method, path, params, headers, body_part}
	return &request, nil
}

func parse_headers(header_part string) map[string]string {
	var headers = make(map[string]string)
	for _, header_string := range strings.Split(header_part, "\r\n") {
		key, value, found := strings.Cut(header_string, ": ")
		if found {
			headers[key] = value
		}
	}

	return headers
}

func append_url_encoded_params(params map[string]string, url_encoded string) {
	for _, query_param := range strings.Split(url_encoded, "&") {
		key, value, found := strings.Cut(query_param, "=")
		if found {
			unescaped_value, err := url.QueryUnescape(value)
			if err != nil {
				fmt.Printf("WARN: invalid query: %s. Reason: %v", value, err)
				params[key] = html.UnescapeString(value)
			} else {
				println("unescaped_value", unescaped_value)
				params[key] = html.UnescapeString(unescaped_value)
			}
		}
	}
}

func mount(method string, path string, handler func(*Request) Response) {
	exist := false
	for _, route := range routes {
		if route.Method == method && route.Path == path {
			exist = true
			break
		}
	}

	if exist {
		println("WARN: handler for route %s %s is already defined", method, path)
	}

	routes = append(routes, Route{method, path, handler})
}

func route(request *Request) Response {
	for _, route := range routes {
		if route.Method == request.Method && route.Path == request.Path {
			return route.Handler(request)
		}
	}

	return Response{Status: 404, Body: []byte("Not found")}
}

func UNUSED(x ...interface{}) {}
