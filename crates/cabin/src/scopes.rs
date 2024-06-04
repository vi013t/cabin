use crate::parser::expressions::{
	util::{name::Name, tags::TagList},
	Expression,
};

use std::{collections::HashMap, fmt::Debug};

#[derive(Clone, Debug)]
/// Information about a variable declaration
pub struct DeclarationData {
	/// The name of the variable
	pub name: Name,
	/// The type of the variable
	pub type_annotation: Option<Expression>,
	/// The value of the variable
	pub value: Option<Expression>,
	/// The tags of the variable
	pub tags: TagList,
}

/// A type of scope in the language. Currently, this is only used for debugging purposes, as scopes are able to be printed as a string representation,
/// and doing so will show their type. However, in the future, this may be used for other purposes, so it's good to leave here regardless
#[derive(Debug)]
pub enum ScopeType {
	/// The function declaration scope type. This is used for the body of a function declaration. Note that this is not in any way related to a scope that
	/// a function is declared in, but represents the scope *inside* of a function's body.
	FunctionDeclaration,

	/// The global scope type. This should only ever be used on a single scope in the whole program: The global scope.
	Global,

	/// The group scope type. This is used for inside group declarations. The only variables that are added to
	/// scopes of this type are compile-time parameters on groups.
	Group,

	/// The block scope type. This is used for the inside of expression blocks.
	Block,
}

/// A scope in the language. This is a node in the overall scope tree that's stored in `ScopeData`. Scopes represent a section of code in curly braces
/// that has variables declared in it which are not accessible outside of it. They are, however, accessible to any child scopes declared within it,
/// meaning that this scope also inherits variables from its parent. One important thing to note is that Cabin doesn't support any kind of shadowing -
/// meaning globally declared variables are available in *every* scope. No matter what scope you're in, you can be 100% certain there is a `String`
/// variable defined, and that it is exactly what you expect it to be. This is important for resolving things like `Boolean`s.
pub struct Scope {
	/// The index of the scope which is the parent to this one. This is the scope's direct parent, i.e., the scope in which this one is declared in. This
	/// is represented as an index into a `ScopeData`'s `scopes` vector, because trying to create a tree data structure in Rust with regular semantics
	/// can get *really* tricky - You need to either resort to lots of unsafe code with raw pointers (and probably pinning), or use some fancy
	/// reference counting wrappers like `Rc<RefCell<Scope>>` and `Weak<RefCell<Scope>>`. Even in doing so, the implementation is not trivial. Using
	/// indices and an arena `Vec` is likely the best option.
	///
	/// This is optional because the global scope has no parent. On all scopes except the global scope, this is guaranteed to be `Some`. On the global scope,
	/// this is guaranteed to be `None`.
	pub parent: Option<usize>,

	/// The indices of the child scopes of this one. This is the scope's direct children, i.e., the scopes in which this one declared them. This
	/// is represented as an index into a `ScopeData`'s `scopes` vector, because trying to create a tree data structure in Rust with regular semantics
	/// can get *really* tricky - You need to either resort to lots of unsafe code with raw pointers (and probably pinning), or use some fancy
	/// reference counting wrappers like `Rc<RefCell<Scope>>` and `Weak<RefCell<Scope>>`. Even in doing so, the implementation is not trivial. Using
	/// indices and an arena `Vec` is likely the best option.
	children: Vec<usize>,

	/// The variables declared in this scope. Note that this only holds the variables declared in this exact specific scope, and does not count the
	/// variables declared in any parent scope, even though those are accessible in the language from this one. To get a variable from anywhere up
	/// the parent tree, use `scope.get_variable`, which will traverse up the scope tree and check all parents.
	variables: HashMap<Name, DeclarationData>,

	/// The index of this scope. This is represented as an index into a `ScopeData`'s `scopes` vector, because trying to create a tree data structure
	/// in Rust with regular semantics can get *really* tricky - You need to either resort to lots of unsafe code with raw pointers (and probably pinning),
	/// or use some fancy reference counting wrappers like `Rc<RefCell<Scope>>` and `Weak<RefCell<Scope>>`. Even in doing so, the implementation is not
	/// trivial. Using indices and an arena `Vec` is likely the best option.
	index: usize,

	/// The type of this scope. This is currently only used for debugging purposes when calling `to_string` on this scope, which will print its type among
	/// other information. However, this may in the future be used for more.
	scope_type: ScopeType,
}

impl Scope {
	/// Returns the information about a variable declared in this scope with the given name. Note that this only checks variables declared exactly
	/// in this scope, and does not check parents of this scope, meaning this cannot give accurate information about whether a variable exists in
	/// the current scope; To get a variable from the current scope, use `get_variable()`, which traverses up the scope tree and recursively checks parents.
	///
	/// # Parameters
	/// - `name` - The name of the variable declared in this scope to get information about
	///
	/// # Returns
	/// A reference to the declaration data of the variable declared in this scope with the given name. If none exists, `None` is returned. If it does exist
	/// and `Some` is returned, the returned reference will have the same lifetime as this `Scope` object.
	pub fn get_variable_direct(&self, name: &Name) -> Option<&DeclarationData> {
		self.variables.get(name)
	}

	/// Returns the information about a variable that exists in this scope with the given name. Note that this will also traverse up the scope tree and check
	/// parent scopes for a variable with the given name if this scope doesn't have a variable with the given name. To get information about a variable that
	/// was declared specifically in this scope, use `get_variable_direct()`. If the function traverses all the way to the global scope and the variable is
	/// not found, `None` is returned.
	///
	/// # Parameters
	/// - `<'scopes>` - The lifetime of the scopes slice passed. The returned declaration reference will have this lifetime.
	/// - `name` - The name of the variable that exists in this scope to get information about
	/// - `scopes` - The scope tree as an area vector slice; This is passed from a `ScopeData` object.
	///
	/// # Returns
	/// A reference to the declaration data of the variable that exists in this scope with the given name. If none exists, `None` is returned. If it does exist
	/// and `Some` is returned, the returned reference will have the same lifetime as this `Scope` object, as well as the given scopes slice.
	pub fn get_variable<'scopes>(&'scopes self, name: &Name, scopes: &'scopes [Self]) -> Option<&DeclarationData> {
		self.variables
			.get(name)
			.or_else(|| self.parent.and_then(|parent| scopes.get(parent).unwrap().get_variable(name, scopes)))
	}

	/// Returns all variables that are available in this scope, including variables declared in ancestor scopes.
	/// This traverses up the scope tree up to and including the global scope, and returns all variables declared
	/// in those scopes.
	///
	/// # Parameters
	/// - `<'scopes>` - The lifetime of the given `Scope` slice. This must be the same as the lifetime of `&self`,
	/// and it will also be the lifetime of the returned references.
	/// - `scopes` - The slice of scopes available from a `ScopeData` object.
	///
	/// # Returns
	/// All variables that exist in this scope, including those declared in ancestor scopes.
	pub fn get_variables<'scopes>(&'scopes self, scopes: &'scopes [Self]) -> Vec<(&'scopes Name, &'scopes DeclarationData)> {
		let mut variables = self.variables.iter().collect::<Vec<_>>();
		if let Some(parent) = self.parent {
			for variable in scopes.get(parent).unwrap().get_variables(scopes) {
				variables.push(variable);
			}
		}

		variables
	}

	/// Reassigns a variable in this scope. This will NOT traverse up the scope tree through the current scope's parents to find the declaration for the given
	/// variable name; it will only reassign a variable declared in this scope. This is only to be used to reassign an existing variable. To add a new variable, l
	/// use `declare_new_variable_direct()`. If no variable with the given name is found, an error is returned.
	///
	/// # Parameters
	/// - `name` - The name of the variable to reassign. A variable with this name must be declared in this scope, otherwise, an `Err` wil be returned.
	/// - `value` - The new value to set the variable to
	///
	/// # Returns
	/// An `Err` if no variable with the given name exists in the current scope.
	fn reassign_variable_direct(&mut self, name: &Name, value: Expression) -> Result<(), Expression> {
		if let Some(variable) = self.variables.get_mut(name) {
			variable.value = Some(value);
			Ok(())
		} else {
			Err(value)
		}
	}

	/// Converts this scope to a debug string representation. This requires the `Scope` slice because it needs to print information about it's children,
	/// which are only stored in the variable as an id (usize) (see the `Scope` struct for reasoning behind this).
	///
	/// # Parameters
	/// - `scopes` - The arena vector of scopes provided by `ScopeData`.
	///
	/// # Returns
	/// A string representation of this scope to debug programs.
	pub fn to_string(&self, scopes: &[Self]) -> String {
		let mut string = vec!["{".to_owned()];
		string.push(format!("\ttype: [{:?}]", self.scope_type));
		string.push(format!("\tindex: [{}]", self.index));
		string.push(format!(
			"\tvariables: [{}],",
			self.variables.keys().map(|name| name.cabin_name()).collect::<Vec<_>>().join(", ")
		));
		for child_scope in &self.children {
			for line in scopes
				.get(*child_scope)
				.unwrap_or_else(|| {
					panic!(
						"Internal Error attempting to convert scope with Id {} to string representation: When looping over the scope's children, a child exists with the id {}, but this is not a valid scope index and does not point to a scope that exists.",
						self.index, child_scope
					)
				})
				.to_string(scopes)
				.lines()
			{
				string.push(format!("\t{line}"));
			}
		}

		string.push("}".to_owned());
		string.join("\n")
	}
}

/// Current scope data for the language. The scopes in the language are a tree data structure, with the root being the global scope. This can be hard to implement
/// in Rust; Some common strategies are using lots of `unsafe` code or using lots of wrapper structures like `Rc<RefCell<Scope>>` and `Weak<RefCell<Scope>>`. Even
/// with these, the implementation is not trivial. The easiest solution is using an "arena allocation" - meaning a single flat vector that holds all scopes,
/// and then "references" to scopes are just indices into that vector. This is a particularly simple pattern to implement here because scopes are never
/// destroyed or removed, so their indices act as permanent unique IDs.
///
/// This acts simply as a wrapper around the scope arena vector, as well as keeping track of the current scope, be it during parsing, compile-time, etc.
pub struct ScopeData {
	/// The arena of scopes stored as a flat vector. For more information, see the documentation on the `ScopeData` struct.
	scopes: Vec<Scope>,
	/// The id of the current scope. This is guaranteed to always point to a valid scope, and by default is the global scope.
	current_scope: usize,
}

impl ScopeData {
	/// Creates a new scope data with an empty global scope. This should only be used once in each program to create the main scope data.
	/// The current scope is set to the global scope.
	///
	/// # Returns
	/// A newly created scope data object with an empty global scope.
	pub fn global() -> Self {
		Self {
			scopes: vec![Scope {
				scope_type: ScopeType::Global,
				index: 0,
				children: Vec::new(),
				variables: HashMap::new(),
				parent: None,
			}],
			current_scope: 0,
		}
	}

	/// Returns an immutable reference to the current scope.
	///
	/// # Returns
	/// An immutable reference to the current scope (did you really have to ask?)
	fn current(&self) -> &Scope {
		self.scopes
			.get(self.current_scope)
			.unwrap_or_else(|| panic!("Internal Error: Context's scope_data's current_scope index does not point to a valid scope in the scope arena."))
	}

	/// Returns a mutable reference to the current scope.
	///
	/// # Returns
	/// A mutable reference to the current scope
	fn current_mut(&mut self) -> &mut Scope {
		self.scopes.get_mut(self.current_scope).unwrap()
	}

	/// Returns a reference to the scope with the given id. If none exists, `None` is returned. This is `O(1)`.
	///
	/// # Parameters
	/// - `id` - The id of the scope to get
	///
	/// # Returns
	/// An immutable reference to the scope with this id, or `None` if no scope exists with the given id.
	pub fn get_scope_from_id(&self, id: usize) -> Option<&Scope> {
		self.scopes.get(id)
	}

	/// Returns the declaration information about a variable that exists in the current scope. The variable may be declared in this scope, or any one of its parents;
	/// As long as it exists in the current scope, the information will be retrieved. If no variable exists in the current scope with the given name,
	/// `None` is returned.
	///
	/// # Parameters
	/// - `name` - The name of the variable to get the information of
	///
	/// # Returns
	/// A reference to the variable declaration, or `None` if the variable does not exist in the current scope.
	pub fn get_variable(&self, name: &Name) -> Option<&DeclarationData> {
		self.current().get_variable(name, &self.scopes)
	}

	/// Returns the declaration information about a variable that exists in the scope with the given id. The variable may be declared in this scope, or any one
	/// of its parents; As long as it exists in the current scope, the information will be retrieved. If no variable exists in the scope with the given id
	/// with the given name, `None` is returned.
	///
	/// # Parameters
	/// - `name` - The name of the variable to get the information of
	///
	/// # Returns
	/// A reference to the variable declaration, or `None` if the variable does not exist in the current scope.
	pub fn get_variable_from_id(&self, name: &Name, id: usize) -> Option<&DeclarationData> {
		self.get_scope_from_id(id).and_then(|scope| scope.get_variable(name, &self.scopes))
	}

	/// Enters a new scope. This creates a new scope with the given scope type, and sets the current scope to be that one. The newly created scope is added
	/// to the children of this scope, and its parent will be this scope. When you're done with this scope, use `exit_scope()`.
	///
	/// # Parameters
	/// - `scope_type` - The type of the scope. For now, this is only used for debugging purposes, but in the future may be used for other things.
	pub fn enter_new_scope(&mut self, scope_type: ScopeType) {
		self.scopes.push(Scope {
			variables: HashMap::new(),
			index: self.scopes.len(),
			parent: Some(self.current_scope),
			children: Vec::new(),
			scope_type,
		});

		let new_id = self.scopes.len() - 1;
		self.current_mut().children.push(new_id);
		self.current_scope = self.scopes.len() - 1;
	}

	/// Exits the current scope. This sets the current scope of this scope data to be the parent of the current scope. This will only return an `Err` if
	/// the current scope is the global scope, which has no parent and should never be exited. This should only ever be used after an accompanying
	/// `enter_new_scope()` call.
	///
	/// # Returns
	/// A result if this is currently the global scope when trying to exit it.
	pub fn exit_scope(&mut self) -> anyhow::Result<()> {
		self.current_scope = self.current().parent.ok_or_else(|| anyhow::anyhow!("Attempted to exit global scope"))?;
		Ok(())
	}

	/// Returns the unique ID of the current scope. This is the index of the current scope in this `ScopeData`'s arena `Scope` vector. Because scopes are never
	/// deleted or removed from this vector, this is a persistent unique ID throughout the duration of the program. This is guaranteed to return a value that
	/// will always be the index of a valid scope (and the same scope that is current when this is called) in the scope vector.
	///
	/// # Returns
	/// The unique ID of the current scope
	pub const fn unique_id(&self) -> usize {
		self.current_scope
	}

	/// Sets the current scope to the given id, and returns the previous scope id. This is used for things like function calls, where the current scope is
	/// temporarily set to the id of the scope inside the function declaration, and then reverted back to the previous scope when returning to the caller.
	///
	/// # Parameters
	/// - `id` - The id of the scope to set
	///
	/// # Returns
	/// The id of the previously current scope
	pub fn set_current_scope(&mut self, id: usize) -> usize {
		let previous = self.current_scope;
		self.current_scope = id;
		previous
	}

	/// Declares a new variable in the scope with the given id with the given value and tags. This should only be used to declare a new variable,
	/// not reassign an existing one. To reassign an existing variable, use `reassign_variable_from_id()`. To declare a new variable in the current scope,
	/// use `declare_new_variable()`.
	///
	/// # Parameters
	/// - `name` - The name of the variable to declare. It must be unused in the scope with the given id, including its parent scopes, or an error will be returned.
	/// - `value` - The value of the variable to set. All variables must be initialized with a value.
	/// - `tags` - The tags on the variable declaration.
	/// - `id` - The id of the scope to declare the variable in.
	///
	/// # Returns
	/// An error if a variable already exists with the given name in the scope with the given id.
	pub fn declare_new_variable_from_id(&mut self, name: Name, type_annotation: Option<Expression>, value: Expression, tags: TagList, id: usize) -> anyhow::Result<()> {
		let mut current = Some(self.current_scope);
		while let Some(current_index) = current {
			if let Some(variable) = self.scopes.get_mut(current_index).unwrap().get_variable_direct(&name) {
				anyhow::bail!("\nError declaring new variable \"{name}\": The variable \"{name}\" already exists in the current scope, and Cabin doesn't allow shadowing\nThe variable is described as follows: {variable:?}", name = name.cabin_name());
			}
			current = self.scopes.get(current_index).unwrap().parent;
		}

		self.scopes.get_mut(id).unwrap().variables.insert(
			name.clone(),
			DeclarationData {
				name,
				type_annotation,
				value: Some(value),
				tags,
			},
		);
		Ok(())
	}

	/// Declares a new variable in the current scope with the given value and tags. This should only be used to declare a new variable,
	/// not reassign an existing one. To reassign an existing variable, use `reassign_variable()`. To declare a new variable in a scope with a specific id,
	/// use `declare_new_variable_from_id()`.
	///
	/// # Parameters
	/// - `name` - The name of the variable to declare. It must be unused in the current scope, including its parent scopes, or an error will be returned.
	/// - `value` - The value of the variable to set. All variables must be initialized with a value.
	/// - `tags` - The tags on the variable declaration.
	///
	/// # Returns
	/// An error if a variable already exists with the given name in the current scope.
	pub fn declare_new_variable(&mut self, name: Name, type_annotation: Option<Expression>, value: Expression, tags: TagList) -> anyhow::Result<()> {
		self.declare_new_variable_from_id(name, type_annotation, value, tags, self.current_scope)
	}

	/// Returns an immutable reference to the global scope in this scope data's scope arena. This does have to traverse up the scope tree,
	/// as only the current scope is stored, so this operation is `O(n)`, where `n` is the height of the scope tree.
	///
	/// # Returns
	/// An immutable reference to the global scope stored in this scope tree.
	fn get_global_scope(&self) -> &Scope {
		let mut current = self.current();
		while let Some(parent_index) = current.parent {
			current = self.scopes.get(parent_index).unwrap();
		}
		current
	}

	/// Reassigns a variable in the scope with the given id. This will traverse up the scope tree through the scope's parents to find the declaration for the given
	/// variable name, and reassign the value. This is only to be used to reassign an existing variable. To add a new variable, use `add_variable()`. To
	/// reassign a variable declared in the current scope, use `reassign_variable()`. If the function traverses all the way into the global scope
	/// and no variable with the given name is found, an error is returned.
	///
	/// # Parameters
	/// - `name` - The name of the variable to reassign. A variable with this name must exist in the current scope, (meaning it's declared here or in a parent
	/// scope), otherwise, an `Err` wil be returned.
	/// - `value` - The new value to set the variable to
	///
	/// # Returns
	/// An `Err` if no variable with the given name exists in the current scope.
	pub fn reassign_variable_from_id(&mut self, name: &Name, mut value: Expression, id: usize) -> anyhow::Result<()> {
		// Traverse up the parent tree looking for the declaration and reassign it
		let mut current = Some(id);
		while let Some(current_index) = current {
			// If we find it, we're done (return Ok), if not, we continue
			match self.scopes.get_mut(current_index).unwrap().reassign_variable_direct(name, value) {
				Ok(()) => return Ok(()),
				Err(returned_value) => value = returned_value,
			}
			current = self.scopes.get(current_index).unwrap().parent;
		}

		// No variable found
		anyhow::bail!(
			"Error reassigning variable \"{name}\": No variable with the name \"{name}\" exists in this scope",
			name = name.cabin_name()
		);
	}

	/// Reassigns a variable in the current scope. This will traverse up the scope tree through the current scope's parents to find the declaration for the given
	/// variable name, and reassign the value. This is only to be used to reassign an existing variable. To add a new variable, use `add_variable()`. To
	/// reassign a variable declared in this specific scope, use `reassign_variable_from_id()`. If the function traverses all the way into the global scope
	/// and no variable with the given name is found, an error is returned.
	///
	/// # Parameters
	/// - `name` - The name of the variable to reassign. A variable with this name must exist in the current scope, (meaning it's declared here or in a parent
	/// scope), otherwise, an `Err` wil be returned.
	/// - `value` - The new value to set the variable to
	///
	/// # Returns
	/// An `Err` if no variable with the given name exists in the current scope.
	pub fn reassign_variable(&mut self, name: &Name, value: Expression) -> anyhow::Result<()> {
		self.reassign_variable_from_id(name, value, self.current_scope)
	}

	/// Returns the variables in the current scope which have the closest names to the given name. This is
	/// used by the compiler to suggest variables with close names when the user attempts to reference a variable
	/// that can't be found.
	///
	/// # Returns
	/// a sorted list, which are sorted by how close they are to the given variable name.
	pub fn get_variables(&self) -> Vec<(&Name, &DeclarationData)> {
		self.current().get_variables(&self.scopes)
	}

	/// Returns the variables in the current scope which have the closest names to the given name. This is
	/// used by the compiler to suggest variables with close names when the user attempts to reference a variable
	/// that can't be found.
	///
	/// # Parameters
	/// - `name` - The name of the variable to get close to
	/// - `max` - The maximum amount of variables to return in the output.
	///
	/// # Returns
	/// a sorted list, of at most `max` elements, which are sorted by how close they are to the given variable
	/// name.
	pub fn get_closest_variables(&self, name: &Name, max: usize) -> Vec<(&Name, &DeclarationData)> {
		let mut all_variables = self.get_variables();
		all_variables.sort_by(|(first, _), (second, _)| first.cabin_name().distance_to(&name.cabin_name()).cmp(&second.cabin_name().distance_to(&name.cabin_name())));

		if all_variables.len() <= max {
			all_variables
		} else {
			all_variables.get(0..max).unwrap().to_vec()
		}
	}

	/// Returns the declaration information of a global variable. This is exactly equivalent to `get_variable_from_id(name, 0)`, because
	/// 0 is always the ID of the global scope.
	///
	/// # Parameters
	/// - `name` - The name of the global variable to get
	///
	/// # Returns
	/// The declaration data for the variable, or `None` if no global variable with this name exists.
	pub fn get_global_variable(&self, name: &Name) -> Option<&DeclarationData> {
		self.get_variable_from_id(name, 0)
	}

	/// Returns the unique ID of the global scope
	///
	/// # Returns
	/// The id of the global scope
	#[allow(clippy::unused_self)]
	pub const fn global_id(&self) -> usize {
		0
	}
}

impl Debug for ScopeData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.get_global_scope().to_string(&self.scopes))
	}
}

/// A trait to apply Levenshtein string distance functionality to strings. This allows strings to calculate how close
/// they are to another string, which is used by the compiler to give suggestions for variables with close names
/// when a variable can't be found.
pub trait Levenshtein {
	/// Returns the Levenshtein distance between this string and another string. This distance is smaller the closer
	/// the two strings are. This is used by the compiler to give suggestions for variables with close names
	/// when a variable can't be found.

	fn distance_to(&self, other: &str) -> usize;
}

impl Levenshtein for String {
	// Algorithm adapted from https://en.wikipedia.org/wiki/Levenshtein_distance#Iterative_with_two_matrix_rows
	fn distance_to(&self, other: &str) -> usize {
		let mut insertion_cost: usize;
		let mut deletion_cost: usize;
		let mut substitution_cost: usize;

		let mut dummy;
		let m = self.len();
		let n = other.len();

		let mut v0 = Vec::new();
		let mut v1 = Vec::new();

		for i in 0..=n {
			v0.push(i);
		}

		for i in 0..m {
			if v1.is_empty() {
				v1.push(0);
			}
			v1[0] = i + 1;

			for j in 0..n {
				deletion_cost = v0[j + 1] + 1;
				insertion_cost = v1[j] + 1;

				substitution_cost = if self.chars().nth(i) == other.chars().nth(j) { v0[j] } else { v0[j] + 1 };

				while v1.len() <= j + 1 {
					v1.push(0);
				}

				v1[j + 1] = [deletion_cost, insertion_cost, substitution_cost].into_iter().min().unwrap();
			}

			dummy = v0;
			v0 = v1;
			v1 = dummy;
		}

		v0[n]
	}
}
