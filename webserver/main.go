package main

import (
	"fmt"
	"html/template"
	"os"
	"strings"
	"time"
)

const host = "127.0.0.1"
const port = "3000"

func main() {
	mount("/", serve_file)
	mount("/index.html", serve_file)
	mount("/info", info_handler)
	server_start(host, port)
}

func info_handler(request *Request) Response {
	data := struct {
		Time    string
		Request *Request
	}{time.Now().String(), request}

	return render_view("info", data)
}

func serve_file(request *Request) Response {
	content, err := os.ReadFile(fmt.Sprintf("www%s", request.Path))
	if err != nil {
		fmt.Printf("WARN: os.ReadFile: %v\n", err)
		return Response{404, []byte("Not found"), nil}
	}

	return Response{200, content, nil}
}

func render_view(view string, data any) Response {
	tmpl, err := os.ReadFile(fmt.Sprintf("www/%s.html.tmpl", view))
	if err != nil {
		fmt.Printf("WARN: os.ReadFile: %v\n", err)
		return Response{404, []byte("Not found"), nil}
	}

	var sb strings.Builder

	t := template.Must(template.New(view).Parse(string(tmpl)))
	err = t.Execute(&sb, data)

	if err != nil {
		fmt.Printf("Error executing template: %v", err)
	}

	return Response{200, []byte(sb.String()), nil}
}
