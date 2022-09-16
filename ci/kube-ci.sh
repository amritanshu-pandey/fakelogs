#!/bin/bash
set -ex

$(dirname ${0})/build-image.sh kubeci
kind load-image faketest:kubeci
kubectl apply -f $(dirname ${0})/../kube/
kubectl get all -n fakelogs 
kubectl delete -f $(dirname ${0})/../kube/
