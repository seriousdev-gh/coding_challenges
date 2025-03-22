package main

import (
	"fmt"
	"html/template"
	"os"
	"strings"
)

func serve_file(filename string) Response {
	content, err := os.ReadFile(fmt.Sprintf("public/%s", filename))
	if err != nil {
		fmt.Printf("WARN: os.ReadFile: %v\n", err)
		return Response{404, []byte("Not found"), nil}
	}

	return Response{200, content, nil}
}

func render_view(view string, data any) Response {
	tmpl, err := os.ReadFile(fmt.Sprintf("views/%s.html.tmpl", view))
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

	fmt.Printf("RENDERED[%v]", sb.String())

	headers := map[string]string{
		"Content-Type": "text/html; charset=utf-8",
	}
	return Response{200, []byte(sb.String()), headers}
}
