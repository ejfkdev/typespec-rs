model Widget {
  id: string;
  weight: int32;
  color: "red" | "blue";
}

@error
model Error {
  code: int32;
  message: string;
}

@route("/widgets")
@tag("Widgets")
interface Widgets {
  list(): Widget[] | Error;
  read(@path id: string): Widget | Error;
  create(@body widget: Widget): Widget | Error;
  update(@path id: string, @body widget: Widget): Widget | Error;
  delete(@path id: string): void | Error;
}
