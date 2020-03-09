use failure::Error;
use std::io::Write;

use crate::parse::{CodeBlockKind, Doc, Node, Tag};

/// Render a `Doc` to the given output target.
pub fn render_document<'a>(doc: &Doc<'a>, output: &mut dyn Write) -> Result<(), Error> {
    println!("{:?}", doc);
    writeln!(output, "")?;
    Renderer::new(output).render_children(doc)?;
    Ok(())
}

#[derive(Debug, Clone)]
enum SepMode {
    Join,
    NewLine,
}

impl SepMode {
    fn render(&self, output: &mut dyn Write) -> Result<(), Error> {
        match self {
            SepMode::Join => (),
            SepMode::NewLine => writeln!(output, "")?,
        }
        Ok(())
    }
}

#[derive(Debug,Clone)]
enum BulletMode {
    Char(char),
    Number(u64),
}

impl BulletMode {
    fn next(&self, opt: &Option<u64>) -> BulletMode {
        if let Some(start) = opt {
            BulletMode::Number(*start)
        } else {
            // TODO: cycle through available bullet types, depending on what self is
            BulletMode::Char('*')
        }
    }

    fn render(&mut self, output: &mut dyn Write) -> Result<(), Error> {
        match self {
            BulletMode::Char(c) => write!(output, "{} ", c)?,

            BulletMode::Number(i) => {
                write!(output, "{}. ", i)?;
                *i += 1;
            }
        }

        Ok(())
    }
}

struct Renderer<'a> {
    sep: SepMode,
    bullet: BulletMode,
    indent: usize,
    output: &'a mut dyn Write,
}

impl<'a> Renderer<'a> {
    fn new(output: &'a mut dyn Write) -> Self {
        Renderer {
            sep: SepMode::NewLine,
            bullet: BulletMode::Char('*'),
            indent: 0,
            output,
        }
    }

    fn render_indent(&mut self) -> Result<(), Error> {
        write!(self.output, "{:indent$}", "", indent=self.indent * 2)?;
        Ok(())
    }

    /// Render a group of children from a single node.
    fn render_children<'b>(&mut self, children: &Doc<'b>) -> Result<(), Error> {
        if let Some(child) = children.first() {
            self.render_node(child)?;

            for child in &children[1..] {
                self.sep.render(self.output)?;
                self.render_node(child)?;
            }
        }

        Ok(())
    }

    /// Render a single node, and all of its children.
    fn render_node<'b>(&mut self, child: &Node<'b>) -> Result<(), Error> {
        match child {
            Node::Node { tag, children } => self.render_nested(tag, children)?,

            Node::Text(cow) => write!(self.output, "{}", &cow)?,

            Node::Code(cow) => write!(self.output, "`{}`", &cow)?,

            Node::Html(cow) => write!(self.output, "{}", &cow)?,

            Node::FootnoteReference(cow) => write!(self.output, "[^{}]", &cow)?,

            Node::SoftBreak => writeln!(self.output, "")?,

            Node::HardBreak => writeln!(self.output, "  ")?,

            Node::Rule => writeln!(self.output, "---")?,

            Node::TaskListMarker(finished) => {
                write!(self.output, "[")?;
                if *finished {
                    write!(self.output, "x")?;
                } else {
                    write!(self.output, " ")?;
                }
                write!(self.output, "] ")?;
            }
        }

        Ok(())
    }

    /// Render a nested node. The tag indicates the type of node that contains the children.
    fn render_nested<'b>(&mut self, tag: &Tag<'b>, children: &Doc<'b>) -> Result<(), Error> {
        match tag {
            Tag::Heading(level) => {
                let sep = self.sep.clone();
                self.sep = SepMode::Join;

                for _ in 0.. 1.max(*level) {
                    write!(self.output, "#")?;
                }

                write!(self.output, " ")?;

                self.render_children(children)?;

                self.sep = sep;
            }

            Tag::List(opt) => {
                let sep = self.sep.clone();
                self.sep = SepMode::NewLine;

                let bullet = self.bullet.clone();
                self.bullet = bullet.next(opt);

                writeln!(self.output, "")?;
                self.render_children(children)?;

                self.sep = sep;
                self.bullet = bullet;
            }

            Tag::Item => {
                let sep = self.sep.clone();
                self.sep = SepMode::Join;

                self.render_indent()?;

                let indent = self.indent;
                self.indent += 1;

                self.bullet.render(self.output)?;
                self.render_children(children)?;

                self.sep = sep;
                self.indent = indent;
            }

            _ => (),
        }

        Ok(())
    }

}
