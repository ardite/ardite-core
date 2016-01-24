//! There are multiple different requests that may be made to a driver. The
//! `Request` enum contains all such requests. This module only contains the
//! `Request` enum.

use values::*;
use structure::Collection;

/// Any driver request. All requests will return an array of documents.
pub enum Request {
  /// The create request. Designed after a [SQL `INSERT` statmenet][1].
  ///
  /// Must return an array of all the created documents with any automatically
  /// generated columns with values.
  ///
  /// [1]: http://www.postgresql.org/docs/current/static/sql-insert.html
  Create {
    /// The collection to create the documents in.
    collection: Collection,
    /// The documents to be created in the database.
    documents: Vec<Document>,
    /// The properties to be returned of the created documents.
    returning: ReturnSet
  },

  /// The read request. Designed after a [SQL `SELECT` statement][1]. Also, a
  /// read request (after much thought) is not recursive/join capable. Instead
  /// join decomposition should be preferred. For more information, see “[big
  /// query v. small query][2]”.
  ///
  /// Must return an array of documents containing the set of documents
  /// described in this request.
  ///
  /// [1]: http://www.postgresql.org/docs/current/static/sql-select.html
  /// [2]: http://dba.stackexchange.com/questions/76973
  Read {
    /// The collection of which to read.
    collection: Collection,
    /// The filter with which to narrow the documents returned.
    filter: Filter,
    /// The order in which to return the documents.
    order: Vec<Ordering>,
    /// A specific range of documents to be read.
    range: Range,
    /// The properties to be returned of the read documents.
    returning: ReturnSet
  },

  /// The update request. Designed after a [SQL `UPDATE` statement][1].
  ///
  /// Must return an array of all the documents that were updated with this
  /// request.
  ///
  /// [1]: http://www.postgresql.org/docs/current/static/sql-update.html
  Update {
    /// The collection in which to update documents.
    collection: Collection,
    /// A filter limiting the number of documents to be updated.
    filter: Filter,
    /// The patches to be applied to the set of documents.
    patches: Vec<Patch>,
    /// The properties to be returned of the updated documents.
    returning: ReturnSet
  },

  /// The delete request. Desinged after a [SQL `DELETE` statement][1].
  ///
  /// Must return an array of all documents that were deleted with this
  /// request.
  ///
  /// [1]: http://www.postgresql.org/docs/current/static/sql-delete.html
  Delete {
    /// The collection to delete documents from.
    collection: Collection,
    /// A filter limiting the number of documents deleted.
    filter: Filter,
    /// What properties to be returned of the deleted documents.
    returning: ReturnSet
  }
}

/// Defines what the request expects to be returned.
pub enum ReturnSet {
  /// All of the properties in the document.
  All,
  /// Only some of the document‘s properties.
  Some(Vec<Property>)
}
