PROJECT_NAME := pgrouting-import
IMAGE_NAME := pgrouting-import
PROJECT_VERSION := $(shell cat VERSION.txt | tr -d '\n')

SHELL := /bin/bash
BOLD := \033[1m
DIM := \033[2m
RESET := \033[0m
RED_BOLD :=\033[0;31m
GREEN_BOLD :=\033[1;32m

EGG := $(shell echo '$(PROJECT_NAME)' | tr - _).egg-info
RENAME:= $(shell which rename)

define success_msg
    (echo -e "$(GREEN_BOLD)$(1)$(RESET)"; true)
endef
define failure_msg
    (echo -e "$(RED_BOLD)$(1)$(RESET)"; $(2))
endef


# ***** DOCKER IMAGE *****

.PHONY: ecr_login
ecr_login:
	@aws ecr get-login-password --region eu-west-1 | docker login --username AWS --password-stdin 268324876595.dkr.ecr.eu-west-1.amazonaws.com

.PHONY: build_release_and_deploy
build_release_and_deploy:
	@docker buildx build --push  \
		-t 268324876595.dkr.ecr.eu-west-1.amazonaws.com/$(IMAGE_NAME):$(PROJECT_VERSION) \
		-t 268324876595.dkr.ecr.eu-west-1.amazonaws.com/$(IMAGE_NAME):latest .


# ***** AWS *****

.PHONY: create_policies_prod
create_policies_prod:
	@aws iam create-policy --policy-name spt-pgrouting-import-s3-policy --policy-document file://aws/prod/s3-policy.json || \
	aws iam create-policy-version --policy-arn=arn:aws:iam::268324876595:policy/spt-pgrouting-import-s3-policy --policy-document file://aws/prod/s3-policy.json --set-as-default && \
	aws iam list-policy-versions --policy-arn arn:aws:iam::268324876595:policy/spt-pgrouting-import-s3-policy --query 'Versions[?IsDefaultVersion==`false`].VersionId' --output text | tr '\t' '\n' | xargs -I {} aws iam delete-policy-version --policy-arn arn:aws:iam::268324876595:policy/spt-pgrouting-import-s3-policy --version-id {}

.PHONY: create_serviceaccount_prod
create_serviceaccount_prod:
	@eksctl create iamserviceaccount --cluster=eks-prod --name=pgrouting-import --namespace=db \
		--role-name pgrouting-import \
		--attach-policy-arn=arn:aws:iam::268324876595:policy/spt-pgrouting-import-s3-policy \
		--override-existing-serviceaccounts --approve


# ***** DEPLOYMENT *****

# Context switch
.PHONY: switch_prod
switch_prod:
	@kubectx singlespot-eks-prod


# ----- Prod -----
.PHONY: deploy_prod
deploy_prod: switch_prod
	@$(if $(shell echo $(PROJECT_VERSION) | sed 's/[.0-9]//g'), $(call failure_msg,"For production deployment version must only contain numbers separated by points", false))
	@$(if $(shell aws ecr batch-get-image --repository-name $(PROJECT_NAME) --image-ids imageTag=$(PROJECT_VERSION)|grep '"ImageNotFound"'),$(call failure_msg,"The current version $(PROJECT_VERSION) is not present on AWS ECR. Please run make build_release_and_deploy first", false))
	@echo -e "$(GREEN_BOLD)Deploying version $(PROJECT_VERSION) for singlespot in production!$(RESET)"
	@$(if $(RENAME), , $(call failure_msg,"Please install rename \'sudo apt install rename\' or \'brew install rename\'", false))
	@sed -i'.bak' 's/{VERSION.txt}/$(PROJECT_VERSION)/' deployment/base/import_job.yml
	@kubectl apply -k deployment/overlays/prod/ ; rename -f  's/\.bak$///' deployment/base/*.yml.bak

.PHONY: undeploy_prod
undeploy_singlespot_prod: switch_prod
	@kubectl delete -k deployment/overlays/prod/