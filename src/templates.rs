pub const HEADER: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="stylesheet" href="/static/style.css">
  </head>
"#;

pub fn render_body(body: &str) -> String {
    format!(
        r#"
        <body>
          <div class="container">
            <div class="header">
                <a href="/static/aboutme.html">Maverick</a>
                Writes
            </div>
            <div class="navigation">
              <ul>
                <li>
                  <a href="/static/aboutme.html">about</a>
                </li>
                <li>
                  <a href="/">blog</a>
                </li>
              </ul>
            </div>

            <br/>

            <div class="body">
            {}
            </div>

        </body>"#,
        body
    )
}

pub const FOOTER: &str = r#"
</html>
"#;