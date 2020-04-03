use moniker::BoundTerm;
use moniker::{Binder, FreeVar, Ignore, Scope, Var};

use pretty::{BoxAllocator, DocAllocator, DocBuilder};
use termcolor::{Color, ColorSpec, WriteColor};

use std::{io::Result, rc::Rc};

use crate::{utils::clone_rc, expr::Expr, flat_expr::FExpr, literals::Literal};

#[derive(Debug, Clone, BoundTerm)]
pub enum UExpr {
    Lam(Scope<Binder<String>, Scope<Binder<String>, Rc<CCall>>>),
    Var(Var<String>),
    Lit(Ignore<Literal>),
}

impl UExpr {
    pub fn pretty<'a, D>(&'a self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            UExpr::Lam(s) => {
                let Scope {
                    unsafe_pattern: pat,
                    unsafe_body:
                        Scope {
                            unsafe_pattern: cont,
                            unsafe_body: body,
                        },
                } = &s;

                let pat_pret = allocator
                    .as_string(pat)
                    .annotate(ColorSpec::new().set_fg(Some(Color::Green)).clone());
                let cont_pret = allocator
                    .as_string(cont)
                    .annotate(ColorSpec::new().set_fg(Some(Color::Red)).clone());
                let args_pret = pat_pret
                    .append(allocator.space())
                    .append(cont_pret)
                    .parens();
                let body_pret = allocator
                    .line_()
                    .append(body.pretty(allocator))
                    .nest(1)
                    .group();

                allocator
                    .text("lambda")
                    .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                    .append(allocator.space())
                    .append(args_pret)
                    .append(allocator.space())
                    .append(body_pret)
                    .parens()
            }
            UExpr::Var(s) => allocator.as_string(s),
            UExpr::Lit(Ignore(l)) => l.pretty(allocator),
        }
    }

    pub fn into_fexpr(self) -> FExpr {
        match self {
            UExpr::Lam(s) => {
                let Scope {
                    unsafe_pattern: pat,
                    unsafe_body:
                        Scope {
                            unsafe_pattern: cont,
                            unsafe_body: body,
                        },
                } = s;

                FExpr::LamTwo(Scope {
                    unsafe_pattern: pat,
                    unsafe_body: Scope {
                        unsafe_pattern: cont,
                        unsafe_body: Rc::new(clone_rc(body).into_fexpr()),
                    },
                })
            }
            UExpr::Var(s) => FExpr::Var(s),
            UExpr::Lit(l) => FExpr::Lit(l),
        }
    }
}

#[derive(Debug, Clone, BoundTerm)]
pub enum KExpr {
    Lam(Scope<Binder<String>, Rc<CCall>>),
    Var(Var<String>),
    Lit(Ignore<Literal>),
}

impl KExpr {
    pub fn pretty<'a, D>(&'a self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            KExpr::Lam(s) => {
                let Scope {
                    unsafe_pattern: pat,
                    unsafe_body: body,
                } = &s;

                let pat_pret = allocator
                    .as_string(pat)
                    .annotate(ColorSpec::new().set_fg(Some(Color::Green)).clone())
                    .parens();
                let body_pret = allocator
                    .line_()
                    .append(body.pretty(allocator))
                    .nest(1)
                    .group();

                allocator
                    .text("lambda")
                    .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                    .append(allocator.space())
                    .append(pat_pret)
                    .append(allocator.space())
                    .append(body_pret)
                    .parens()
            }
            KExpr::Var(s) => allocator.as_string(s),
            KExpr::Lit(Ignore(l)) => l.pretty(allocator),
        }
    }

    pub fn into_fexpr(self) -> FExpr {
        match self {
            KExpr::Lam(s) => {
                let Scope {
                    unsafe_pattern: pat,
                    unsafe_body: body,
                } = s;

                FExpr::LamOne(Scope {
                    unsafe_pattern: pat,
                    unsafe_body: Rc::new(clone_rc(body).into_fexpr()),
                })
            }
            KExpr::Var(s) => FExpr::Var(s),
            KExpr::Lit(l) => FExpr::Lit(l),
        }
    }
}

#[derive(Debug, Clone, BoundTerm)]
pub enum CCall {
    UCall(Rc<UExpr>, Rc<UExpr>, Rc<KExpr>),
    KCall(Rc<KExpr>, Rc<UExpr>),
}

impl CCall {
    pub fn pretty<'a, D>(&'a self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            CCall::UCall(f, v, c) => {
                let f_pret = f.pretty(allocator);
                let v_pret = v.pretty(allocator);
                let c_pret = c.pretty(allocator);

                f_pret
                    .annotate(ColorSpec::new().set_fg(Some(Color::Blue)).clone())
                    .append(allocator.space())
                    .append(v_pret)
                    .append(allocator.space())
                    .append(c_pret)
                    .parens()
            }

            CCall::KCall(f, c) => {
                let f_pret = f.pretty(allocator);
                let c_pret = c.pretty(allocator);

                f_pret
                    .annotate(ColorSpec::new().set_fg(Some(Color::Blue)).clone())
                    .append(allocator.space())
                    .append(c_pret)
                    .parens()
            }
        }
    }

    pub fn pretty_print(&self, out: impl WriteColor) -> Result<()> {
        let allocator = BoxAllocator;

        self.pretty(&allocator).1.render_colored(70, out)?;

        Ok(())
    }

    pub fn into_fexpr(self) -> FExpr {
        match self {
            CCall::UCall(f, v, c) => FExpr::CallTwo(
                Rc::new(clone_rc(f).into_fexpr()),
                Rc::new(clone_rc(v).into_fexpr()),
                Rc::new(clone_rc(c).into_fexpr()),
            ),
            CCall::KCall(f, v) => FExpr::CallOne(
                Rc::new(clone_rc(f).into_fexpr()),
                Rc::new(clone_rc(v).into_fexpr()),
            ),
        }
    }
}

pub fn t_k(expr: Expr, k: Rc<KExpr>) -> CCall {
    match expr {
        e @ (Expr::Lam(_) | Expr::Var(_) | Expr::Lit(_)) => CCall::KCall(k, Rc::new(m(e))),
        Expr::App(f, e) => {
            let rv_v = FreeVar::fresh_named("rv");
            let cont = Rc::new(KExpr::Lam(Scope::new(
                Binder(rv_v.clone()),
                Rc::new(CCall::KCall(k, Rc::new(UExpr::Var(Var::Free(rv_v))))),
            )));

            let f_v = FreeVar::fresh_named("f");
            let e_v = FreeVar::fresh_named("e");

            t_k(
                clone_rc(f),
                Rc::new(KExpr::Lam(Scope::new(
                    Binder(f_v.clone()),
                    Rc::new(t_k(
                        clone_rc(e),
                        Rc::new(KExpr::Lam(Scope::new(
                            Binder(e_v.clone()),
                            Rc::new(CCall::UCall(
                                Rc::new(UExpr::Var(Var::Free(f_v))),
                                Rc::new(UExpr::Var(Var::Free(e_v))),
                                cont,
                            )),
                        ))),
                    )),
                ))),
            )
        }
    }
}

fn t_c(expr: Expr, c: FreeVar<String>) -> CCall {
    let c_v = Rc::new(KExpr::Var(Var::Free(c)));
    match expr {
        e @ (Expr::Lam(_) | Expr::Var(_) | Expr::Lit(_)) => CCall::KCall(c_v, Rc::new(m(e))),
        Expr::App(f, e) => {
            let f_v = FreeVar::fresh_named("f");
            let e_v = FreeVar::fresh_named("e");

            t_k(
                clone_rc(f),
                Rc::new(KExpr::Lam(Scope::new(
                    Binder(f_v.clone()),
                    Rc::new(t_k(
                        clone_rc(e),
                        Rc::new(KExpr::Lam(Scope::new(
                            Binder(e_v.clone()),
                            Rc::new(CCall::UCall(
                                Rc::new(UExpr::Var(Var::Free(f_v))),
                                Rc::new(UExpr::Var(Var::Free(e_v))),
                                c_v,
                            )),
                        ))),
                    )),
                ))),
            )
        }
    }
}

fn m(expr: Expr) -> UExpr {
    match expr {
        Expr::Lam(s) => {
            let (p, t) = s.unbind();
            let k = FreeVar::fresh_named("k");
            let body = t_c(clone_rc(t), k.clone());
            UExpr::Lam(Scope::new(p, Scope::new(Binder(k), Rc::new(body))))
        }
        Expr::Var(v) => UExpr::Var(v),
        Expr::Lit(v) => UExpr::Lit(v),
        _ => unreachable!(),
    }
}
