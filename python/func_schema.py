from sqlalchemy import ForeignKey
from sqlalchemy.orm import relationship
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy import Column, Integer, Enum, String, Boolean
from sqlalchemy.ext.declarative import declared_attr

import enum

Base = declarative_base()
engine = None
Session = None

class BaseMixin:
    @declared_attr
    def __tablename__(cls):
        return cls.__name__.lower()

    id = Column(Integer, primary_key=True)


class AccessModifier(enum.Enum):
    PRIVATE = enum.auto()
    PUBLIC = enum.auto()
    PROTECTED = enum.auto()

    @classmethod
    def from_string(cls, string):
        string = string.upper()
        if string == 'PRIVATE':
            return cls.PRIVATE
        elif string == 'PUBLIC':
            return cls.PUBLIC
        else:
            return cls.PROTECTED


class FunctionArg(BaseMixin, Base):
    name = Column(String)
    crab_type = Column(String)
    nullable = Column(Boolean)
    reference = Column(Boolean)
    parent_id = Column(Integer, ForeignKey('function.id'))

    def __init__(self, arg_dict):
        self.name = arg_dict['name']
        self.crab_type = arg_dict['typed']['name']
        self.nullable = arg_dict['typed']['nullable']
        self.reference = arg_dict['typed']['reference']


class FunctionReturn(BaseMixin, Base):
    crab_type = Column(String)
    nullable = Column(Boolean)
    parent_id = Column(Integer, ForeignKey('function.id'))

    def __init__(self, return_dict):
        # TODO: implement
        pass


class Namespace(BaseMixin, Base):
    name = Column(String)
    parent_id = Column(Integer, ForeignKey('function.id'))

    def __init__(self, ns):
        self.name = ns

class Function(BaseMixin, Base):
    access_modifier = Column(Enum(AccessModifier))
    name = Column(String)
    errable = Column(Boolean)
    args = relationship('FunctionArg')
    returns = relationship('FunctionReturn')
    namespace = relationship('Namespace')

    def __init__(self, function_dict, namespace):
        self.name = function_dict['name']
        self.access_modifier = AccessModifier.from_string(function_dict['access_modifier'])
        self.errable = function_dict['errable']
        for arg_dict in function_dict['args']:
            self.args.append(FunctionArg(arg_dict))
        for returns_dict in function_dict['returns']:
            self.returns.append(FunctionReturn(returns_dict))
        for ns in namespace:
            self.namespace.append(Namespace(ns))
