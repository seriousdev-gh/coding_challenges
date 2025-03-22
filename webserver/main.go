package main

import (
	"time"
)

const host = "127.0.0.1"
const port = "3000"

func main() {
	mount("GET", "/", index_page)
	mount("GET", "/form", form_page)
	mount("POST", "/form", form_page_create)
	mount("GET", "/info", info_page)
	server_start(host, port)
}

func index_page(_ *Request) Response {
	return serve_file("index.html")
}

func form_page(_ *Request) Response {
	return serve_file("form.html")
}

func form_page_create(request *Request) Response {
	return render_view("form_submitted", request.Params)
}

func info_page(request *Request) Response {
	data := struct {
		Time    string
		Request *Request
	}{
		time.Now().String(),
		request,
	}

	return render_view("info", data)
}
