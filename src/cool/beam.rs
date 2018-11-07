use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use crate::parser::*;
use crate::lexer::Token;

// macro_rules! check_default {
// 	($id: ident, $expr: expr, $default: expr) => {
// 		let $id = if $id == $expr { $default } else { $id };
// 	};
// 	(&$id: ident, $expr: expr, $default: expr) => {
// 		let $id = if $id == $expr { $default } else { $id.clone() };
// 	};
// }

pub type CoolIdentifier = String;
pub type CoolTypename = String;

pub trait CoolAstNode {
	fn parse(node: &Node) -> Self;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CoolOperator {
	Add, Subtract, Multiply, Divide, Negate, CmpLt, CmpEq, CmpLeq, Not
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoolExpressionImpl {
	Integer(i64), String(String), Bool(bool), Void,
	Assignment(CoolIdentifier, Box<CoolExpression>),
	Block(Vec<CoolExpression>),
	If(Box<CoolExpression>, Box<CoolExpression>, Box<CoolExpression>),
	While(Box<CoolExpression>, Box<CoolExpression>),
	Case(Box<CoolExpression>, Vec<CoolAttribute>),
	New(CoolTypename),
	Isvoid(Box<CoolExpression>),
	Operation(CoolOperator, Vec<CoolExpression>),
	Object(CoolIdentifier),
	Let(Vec<CoolAttribute>, Box<CoolExpression>),
	Dispatch(Box<CoolExpression>, Option<CoolTypename>, CoolIdentifier, Vec<CoolExpression>),
	Tuple(Box<CoolExpression>),
}


#[derive(Debug, Clone, PartialEq)]
pub struct CoolExpression {
	pub impl_: CoolExpressionImpl,
	pub type_: CoolTypename,
	// pub temporary_: bool,
	pub pos_: CoolPosition,
}

impl Default for CoolExpression {
	fn default() -> Self {
		CoolExpression {
			impl_: CoolExpressionImpl::Void,
			type_: CoolTypename::default(),
			// temporary_: false,
			pos_: CoolPosition::default(),
		}
	}
}

impl CoolAstNode for CoolExpression {
	fn parse(node: &Node) -> Self {
		let mut exprs: Vec<CoolExpression> = node.childs
			.iter()
			.filter_map(|x| {
				if let NodeType::NonTerminal(NonTerminal { ref type_, .. }) = x.value {
					if type_ == "expr" {
						return Some(CoolExpression::parse(x));
					}
				}
				None
			}).collect();
		let impl_ = if let NodeType::NonTerminal(NonTerminal { ref value_, .. }) = node.value {
			match value_.as_str() {
				"add" => CoolExpressionImpl::Operation(CoolOperator::Add, exprs),
				"sub" => CoolExpressionImpl::Operation(CoolOperator::Subtract, exprs),
				"mul" => CoolExpressionImpl::Operation(CoolOperator::Multiply, exprs),
				"div" => CoolExpressionImpl::Operation(CoolOperator::Divide, exprs),
				"neg" => CoolExpressionImpl::Operation(CoolOperator::Negate, exprs),
				"lt" => CoolExpressionImpl::Operation(CoolOperator::CmpLt, exprs),
				"le" => CoolExpressionImpl::Operation(CoolOperator::CmpLeq, exprs),
				"eq" => CoolExpressionImpl::Operation(CoolOperator::CmpEq, exprs),
				"not" => CoolExpressionImpl::Operation(CoolOperator::Not, exprs),
				"isvoid" => CoolExpressionImpl::Isvoid(Box::new(exprs.remove(0))),
				"tuple" => CoolExpressionImpl::Tuple(Box::new(exprs.remove(0))),
				"if" => CoolExpressionImpl::If(Box::new(exprs.remove(0)), Box::new(exprs.remove(0)), Box::new(exprs.remove(0))),
				"while" => CoolExpressionImpl::While(Box::new(exprs.remove(0)), Box::new(exprs.remove(0))),
				"let" => {
					let mut attrs: Vec<CoolAttribute> = Vec::new();
					for x in &node.childs[..node.childs.len()-1] {
						if let NodeType::Terminal(Token { ref type_, ref value_, .. }) = x.value {
							if type_ == "OBJECTID" {
								attrs.push(CoolAttribute {
									name_: value_.clone(),
									..Default::default()
								});
							} else if type_ == "TYPEID" {
								attrs.last_mut().unwrap().type_ = value_.clone();
							}
						} else if let NodeType::NonTerminal(NonTerminal { ref type_, .. }) = x.value {
							if type_ == "expr" {
								attrs.last_mut().unwrap().value_ = CoolExpression::parse(x);
							}
						}
					}
					CoolExpressionImpl::Let(attrs, Box::new(exprs.pop().unwrap()))
				},
				"case" => {
					let objs: Vec<String> = node.childs
						.iter()
						.filter_map(|x| {
							if let NodeType::Terminal(Token { ref type_, ref value_, .. }) = x.value {
								if type_ == "OBJECTID" {
									return Some(value_.clone());
								}
							}
							None
						}).collect();
					let tys: Vec<String> = node.childs
						.iter()
						.filter_map(|x| {
							if let NodeType::Terminal(Token { ref type_, ref value_, .. }) = x.value {
								if type_ == "TYPEID" {
									return Some(value_.clone());
								}
							}
							None
						}).collect();
					CoolExpressionImpl::Case(Box::new(exprs.remove(0)), objs
						.into_iter()
						.zip(tys.into_iter())
						.zip(exprs.into_iter())
						.map(|((obj, ty), expr)| CoolAttribute {
							name_: obj,
							type_: ty,
							pos_: expr.pos_.clone(),
							value_: expr,
						}).collect())
				},
				"block" => CoolExpressionImpl::Block(exprs),
				"int" => CoolExpressionImpl::Integer(if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[0].value {
					value_.parse().unwrap()
				} else { panic!() }),
				"string" => CoolExpressionImpl::String(if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[0].value {
					value_.clone()
				} else { panic!() }),
				"bool" => CoolExpressionImpl::Bool(if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[0].value {
					value_.parse().unwrap()
				} else { panic!() }),			
				"object" => CoolExpressionImpl::Object(if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[0].value {
					value_.clone()
				} else { panic!() }),
				"assign" => CoolExpressionImpl::Assignment(if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[0].value {
					value_.clone()
				} else { panic!() }, Box::new(exprs.remove(0))),
				"dispatch" => {
					let at = if let NodeType::Terminal(Token { ref type_, .. }) = &node.childs[1].value {
						value_ == "TYPE_ANN"
					} else { false };
					if at {
						CoolExpressionImpl::Dispatch(Box::new(exprs.remove(0)), Some(if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[2].value {
							value_.clone()
						} else { panic!() }), if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[4].value {
							value_.clone()
						} else { panic!() }, exprs)
					} else {
						CoolExpressionImpl::Dispatch(Box::new(exprs.remove(0)), None, if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[2].value {
							value_.clone()
						} else { panic!() }, exprs)
					}
				},
				"sdispatch" => CoolExpressionImpl::Dispatch(Box::new(CoolExpression {
					impl_: CoolExpressionImpl::Object(String::from("self")),
					pos_: CoolPosition::from(node),
					..Default::default()
				}), None, if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[0].value {
					value_.clone()
				} else { panic!() }, exprs),
				"new" => CoolExpressionImpl::New(if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[0].value {
						value_.clone()
				} else { panic!() }),	
				_ => { panic!(format!("Unknown expression type : {:?}", node)) }
			}
		} else { panic!("Inner error") };
		CoolExpression {
			impl_: impl_,
			pos_: CoolPosition::from(node),
			..Default::default()
		}
	}
}


impl CoolExpression {
	// pub fn get_associated_class(&self) -> Option<CoolClass> {
	// 	None
	// }

	// pub fn get_associated_method(&self) -> Option<CoolMethod> {
	// 	None
	// }

	// pub fn get_associated_attribute(&self) -> Option<CoolAttribute> {
	// 	None
	// }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CoolAttribute {
	pub name_: CoolIdentifier,
	pub type_: CoolTypename,
	pub value_: CoolExpression,

	pub pos_: CoolPosition,
}

impl CoolAstNode for CoolAttribute {
	fn parse(node: &Node) -> Self {
		let mut ret = CoolAttribute::default();
		ret.pos_ = CoolPosition::from(node);
		if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[0].value {
			ret.name_ = value_.clone();
		}
		if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[2].value {
			ret.type_ = value_.clone();
		}
		if let Some(n) = node.childs.get(4) {
			if let NodeType::NonTerminal(NonTerminal { ref type_, .. }) = n.value {
				if type_ == "expr" {
					ret.value_ = CoolExpression::parse(n);
				}
			}
		}
		ret
	}
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CoolParameter {
	pub name_: CoolIdentifier,
	pub type_: CoolTypename,
	// TODO: default value

	pub pos_: CoolPosition,
}

impl CoolAstNode for CoolParameter {
	fn parse(node: &Node) -> Self {
		let mut ret = CoolParameter::default();
		ret.pos_ = CoolPosition::from(node);
		if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[0].value {
			ret.name_ = value_.clone();
		}
		if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[2].value {
			ret.type_ = value_.clone();
		}
		ret
	}
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CoolMethod {
	pub name_: CoolIdentifier,
	pub type_: CoolTypename,
	pub parameters_: Vec<CoolParameter>,
	pub expression_ : Box<CoolExpression>,

	pub external_: bool,
	pub pos_: CoolPosition,
}

impl CoolAstNode for CoolMethod {
	fn parse(node: &Node) -> Self {
		let mut ret = CoolMethod::default();
		ret.pos_ = CoolPosition::from(node);
		if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[0].value {
			ret.name_ = value_.clone();
		}
		for child in &node.childs {
			if let NodeType::NonTerminal(NonTerminal { ref type_, ref value_, .. }) = child.value {
				if type_ == "formal" {
					ret.parameters_.push(CoolParameter::parse(&child));
				} else if type_ == "expr" {
					ret.expression_ = Box::new(CoolExpression::parse(&child));
				}
			}
		}
		ret
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoolClass {
	pub name_: CoolTypename,
	pub inherit_: CoolTypename,
	pub attributes_: Vec<CoolAttribute>,
	pub methods_: Vec<CoolMethod>,

	pub pos_: CoolPosition,
}

impl Default for CoolClass {
	fn default() -> Self {
		CoolClass {
			name_: CoolTypename::default(),
			inherit_: CoolEnvironment::object_type(),
			attributes_: Vec::default(),
			methods_: Vec::default(),
			pos_: CoolPosition::default(),
		}
	}
}

impl CoolAstNode for CoolClass {
	fn parse(node: &Node) -> Self {
		let mut ret = CoolClass::default();
		ret.pos_ = CoolPosition::from(node);
		if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[1].value {
			ret.name_ = value_.clone();
		}
		if let NodeType::Terminal(Token { ref type_, .. }) = &node.childs[2].value {
			if type_ == "INHERITS" {
				if let NodeType::Terminal(Token { ref value_, .. }) = &node.childs[3].value {
					ret.inherit_ = value_.clone();
				}
			}
		}
		for child in &node.childs {
			if let NodeType::NonTerminal(NonTerminal { ref type_, ref value_, .. }) = child.value {
				if type_ == "feature" && value_ == "attribute" {
					ret.attributes_.push(CoolAttribute::parse(&child));
				} else if type_ == "feature" && value_ == "method" {
					ret.methods_.push(CoolMethod::parse(&child));
				}
			}
		}
		ret
	}
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CoolFile {
	pub classes_: Vec<CoolClass>,
}

impl CoolAstNode for CoolFile {
	fn parse(node: &Node) -> Self {
		let mut ret = CoolFile::default();
		if let NodeType::NonTerminal(NonTerminal { ref type_, .. }) = node.value {
			if type_ == "program" {
				for child in &node.childs {
					if let NodeType::NonTerminal(NonTerminal { ref type_, .. }) = child.value {
						if type_ == "class" {
							ret.classes_.push(CoolClass::parse(&child));
						}
					}
				}
			}

		}
		ret
	}
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CoolProgram {
	pub classes_: HashMap<CoolTypename, CoolClass>,
}

impl CoolProgram {
	pub fn add_file(&mut self, f: CoolFile) -> Result<(), CoolCompileError> {
		for cls in f.classes_ {
			let name = cls.name_.clone();
			if let Some(cls) = self.classes_.insert(name, cls) {
				return Err(CoolCompileError::new(format!("Multiply definition for class {:}", cls.name_)))
			}
		}
		Ok(())
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct CoolPosition {
	pub lines: usize,
	pub chars: usize,
}

impl<'a> From<&'a Node> for CoolPosition {
	fn from(node: &'a Node) -> Self {
		CoolPosition {
			lines: node.index,
			chars: 0,
		}
	}
}

pub type CoolClassInheritMap = HashMap<CoolTypename, Vec<CoolTypename>>;


#[derive(Debug, Clone, PartialEq, Default)]
pub struct CoolEnvironment{
	pub objects_: HashMap<CoolIdentifier, CoolTypename>,
	pub methods_: HashMap<CoolTypename, HashMap<CoolIdentifier, Vec<CoolTypename>>>,
	pub self_type_: CoolTypename,
	pub inherits_ : HashMap<CoolTypename, CoolTypename>,
}

impl CoolEnvironment {
	
	pub fn string_type() -> CoolTypename {
		"String".to_string()
	}

	pub fn integer_type() -> CoolTypename {
		"Int".to_string()
	}

	pub fn dynamic_type() -> CoolTypename {
		"$Unknown".to_string()
	}

	pub fn pointer_type() -> CoolTypename {
		"$Pointer".to_string()
	}

	pub fn bool_type() -> CoolTypename {
		"Bool".to_string()
	}

	pub fn object_type() -> CoolTypename {
		"Object".to_string()
	}

	pub fn self_type() -> CoolTypename {
		"SELF_TYPE".to_string()
	}

	pub fn self_object() -> CoolIdentifier {
		"self".to_string()
	}

	pub fn get_object(&self, id: &CoolIdentifier) -> Result<CoolTypename, CoolCompileError> {
		if let Some(t) = self.objects_.get(id) {
			Ok(t.clone())
		} else {
			Err(CoolCompileError::new(id.as_ref()))
		}
	}

	pub fn get_method(&self, cls: &CoolTypename, id: &CoolIdentifier) -> Result<Vec<CoolTypename>, CoolCompileError> {
		let mut cls = if cls == &CoolEnvironment::self_type() { self.self_type_.clone() } else { cls.clone() };
		loop {
			if let Some(ms) = self.methods_.get(&cls) {
				if let Some(t) = ms.get(id) {
					return Ok(t.clone())
				} else {
					if cls == CoolEnvironment::object_type() { break; }
					if let Ok(c) = self.get_inherited(&cls) {
						cls = c;
					} else { break; }
				}
			} else { break; }
		}
		Err(CoolCompileError::new(format!("Method not found: {:} :: {:}", cls, id)))
	}

	pub fn get_inherited(&self, cls: &CoolTypename) -> Result<CoolTypename, CoolCompileError> {
		if let Some(t) = self.inherits_.get(cls) {
			Ok(t.clone())
		} else {
			Err(CoolCompileError::new(cls.as_ref()))
		}
	}

	pub fn get_self_type(&self) -> CoolTypename {
		self.self_type_.clone()
	}

	pub fn set_object(&mut self, id: &CoolIdentifier, ty: &CoolTypename) {
		self.objects_.insert(id.clone(), ty.clone());
	}

	pub fn is_type(&self, ty: &CoolTypename, tys: &CoolTypename) -> bool {
		check_default!(&ty, &CoolEnvironment::self_type(), self.get_self_type());
		check_default!(&tys, &CoolEnvironment::self_type(), self.get_self_type());
		if ty == tys || ty == CoolEnvironment::dynamic_type() { return true; }
		let mut ty = ty.clone();
		loop {
			if ty == CoolEnvironment::object_type() { break; }
			if let Ok(t) = self.get_inherited(&ty) {
				ty = t;
				if ty == tys { return true; }
			} else { break; }
		}
		false
	}

	pub fn join_type(&self, ty1: &CoolTypename, ty2: &CoolTypename) -> CoolTypename {
		if ty1 == &CoolTypename::default() { return ty2.clone() }
		if ty2 == &CoolTypename::default() { return ty1.clone() }
		check_default!(&ty1, &CoolEnvironment::self_type(), self.get_self_type());
		check_default!(&ty2, &CoolEnvironment::self_type(), self.get_self_type());
		let mut map = Vec::new();
		let mut ty = ty1;
		map.push(ty.clone());
		loop {
			if ty == CoolEnvironment::object_type() { break; }
			if let Ok(t) = self.get_inherited(&ty) {
				ty = t;
				map.push(ty.clone())
			} else { panic!("No super class for {:?}", ty); }
		}
		let mut ty = ty2;
		loop {
			if map.contains(&ty) {
				return ty.clone();
			}
			else if let Ok(t) = self.get_inherited(&ty) {
				ty = t;
			} else { panic!("No super class for {:?}", ty); }
		}

	}

}


// Specify Compile Error

#[derive(Debug)]
pub struct CoolCompileError {
	msg: String,
	index: usize
}

impl CoolCompileError {
	pub fn new<S: Into<String>>(s: S) -> Self {
		CoolCompileError {
			msg: s.into(), 
			index: 0
		}
	}
}

impl fmt::Display for CoolCompileError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl Error for CoolCompileError {
}

