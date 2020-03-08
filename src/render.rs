use failure::Error;
use std::io::Write;

use crate::parse::{CodeBlockKind, Doc, Node, Tag};

#[derive(Clone)]
enum Bullet {
    Numbered(u64),
    Char(char),
}

pub struct Renderer<'a> {
    level: usize,
    bullet: Bullet,
    output: &'a mut dyn Write,
}

impl<'a> Renderer<'a> {
    pub fn new(output: &'a mut dyn Write) -> Self {
        Renderer {
            level: 0,
            bullet: Bullet::Char('*'),
            output,
        }
    }

    pub fn render_doc<'b>(&mut self, doc: &Doc<'b>) -> Result<(), Error> {
        for node in doc {
            self.render_node(node)?;
        }
        Ok(())
    }

    fn indent(&self) -> usize {
        if self.level > 1 {
            (self.level - 1) * 2
        } else {
            0
        }
    }

    fn render_node<'b>(&mut self, node: &Node<'b>) -> Result<(), Error> {
        match node {
            Node::Node { tag, children } => {
                self.render_nested(tag, children)?;
            }
            Node::Text(cow) => {
                write!(self.output, "{}", &cow)?;
            }
            Node::Code(cow) => {
                write!(self.output, "`{}`", &cow)?;
            }
            Node::SoftBreak => {
                writeln!(self.output, "")?;
            }
            Node::HardBreak => {
                writeln!(self.output, "  ")?;
            }
            Node::Rule => {
                writeln!(self.output, "---")?;
                writeln!(self.output, "")?;
            }
            Node::TaskListMarker(b) => {
                write!(self.output, "[")?;
                if *b {
                    write!(self.output, "x")?;
                } else {
                    write!(self.output, " ")?;
                }
                write!(self.output, "] ")?;
            }
            _ => (),
        }

        Ok(())
    }

    fn render_nested<'b>(&mut self, tag: &Tag<'b>, children: &Vec<Node<'b>>) -> Result<(), Error> {
        match tag {
            Tag::Heading(level) => {
                for _ in 0..*level {
                    write!(self.output, "#")?;
                }

                write!(self.output, " ")?;

                self.render_doc(children)?;
            }
            Tag::Paragraph => {
                self.render_doc(children)?;
            }
            Tag::List(opt) => {
                let saved = self.level;
                self.level += 1;
                self.render_list(*opt, children)?;
                self.level = saved;
            }
            Tag::Item => {
                self.render_bullet()?;
                self.render_doc(children)?;
            }
            Tag::CodeBlock(CodeBlockKind::Fenced(lang)) => {
                writeln!(self.output, "```{}", &lang)?;
                self.render_doc(children)?;
                writeln!(self.output, "```")?;
            }

            Tag::CodeBlock(CodeBlockKind::Indented) => {
                let saved = self.level;
                self.level = 2;
                self.render_doc(children)?;
                self.level = saved;
            }

            _ => (),
        }

        writeln!(self.output, "")?;

        Ok(())
    }

    fn render_list<'b>(&mut self, opt: Option<u64>, children: &Vec<Node<'b>>) -> Result<(), Error> {
        let saved = self.bullet.clone();
        self.bullet = if let Some(index) = opt {
            Bullet::Numbered(index)
        } else {
            Bullet::Char('*')
        };

        self.render_doc(children)?;

        self.bullet = saved;

        Ok(())
    }

    fn render_bullet(&mut self) -> Result<(), Error> {
        let indent = self.indent();
        match self.bullet {
            Bullet::Numbered(ref mut val) => {
                write!(
                    self.output,
                    "{:indent$}{}. ",
                    "",
                    val,
                    indent = indent
                )?;
                *val += 1;
            }

            Bullet::Char(c) => {
                write!(self.output, "{:indent$}{} ", "", c, indent = indent)?;
            }
        }

        Ok(())
    }
}
