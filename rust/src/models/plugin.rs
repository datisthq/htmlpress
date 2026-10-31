use textlane::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::code::annotate::annotate_code;
use crate::actions::math::wrap::wrap_math;

/// htmlpress' own step of the page pipeline, between textlane's `html.math` and
/// `html.katex`: every formula into its `<htmlpress-math>` (see `wrap_math`).
#[derive(Clone, Debug, Default)]
pub(crate) struct WrapMath;

impl Plugin for WrapMath {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(WrapMath));
        Ok(())
    }
}

impl Transform for WrapMath {
    fn input(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        match tree {
            Tree::Hast(mut node) => {
                wrap_math(&mut node, alloc);
                Ok(Tree::Hast(node))
            }
            _ => Err(Error::new(
                "invalid-stage",
                "htmlpress' math step takes hast",
            )),
        }
    }
}

/// htmlpress' own step after `md.toHtml`: a fence's meta onto its block's `data-*`
/// attributes (see `annotate_code`).
#[derive(Clone, Debug, Default)]
pub(crate) struct AnnotateCode;

impl Plugin for AnnotateCode {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(AnnotateCode));
        Ok(())
    }
}

impl Transform for AnnotateCode {
    fn input(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        match tree {
            Tree::Hast(mut node) => {
                annotate_code(&mut node, alloc);
                Ok(Tree::Hast(node))
            }
            _ => Err(Error::new(
                "invalid-stage",
                "htmlpress' code step takes hast",
            )),
        }
    }
}
